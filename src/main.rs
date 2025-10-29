use anyhow::Result;
use clap::Parser;
use rust_htslib::{bam, bam::Read};
use std::collections::HashMap;

const KMER_SIZE: usize = 21;

#[derive(Parser, Debug)]
#[command(name = "filter_bam_pairs")]
#[command(about = "Filter paired-end BAM reads by kmer complexity and mapped bases", long_about = None)]
struct Args {
    /// Input BAM file (must be name-sorted)
    #[arg(short, long, value_name = "FILE")]
    input: String,

    /// Output BAM file
    #[arg(short, long, value_name = "FILE")]
    output: String,

    /// Kmer complexity cutoff (0.0-1.0, default: 0.8)
    #[arg(short, long, default_value = "0.8")]
    complexity: f64,

    /// Minimum contiguous mapped bases (default: 0 = disabled)
    #[arg(short, long, default_value = "0")]
    min_mapped: u32,
}

/// Calculate kmer complexity: unique_kmers / total_kmers
fn calculate_kmer_complexity(sequence: &[u8]) -> f64 {
    if sequence.len() < KMER_SIZE {
        return 0.0;
    }

    let mut kmer_counts: HashMap<&[u8], u32> = HashMap::new();
    let total_kmers = sequence.len() - KMER_SIZE + 1;

    // Extract and count kmers
    for i in 0..=sequence.len() - KMER_SIZE {
        let kmer = &sequence[i..i + KMER_SIZE];
        *kmer_counts.entry(kmer).or_insert(0) += 1;
    }

    let unique_kmers = kmer_counts.len() as f64;
    unique_kmers / total_kmers as f64
}

/// Get longest contiguous mapped bases from CIGAR
fn get_longest_mapped_bases(record: &bam::Record) -> u32 {
    let mut longest = 0u32;
    let mut current = 0u32;

    for cigar_op in record.cigar().iter() {
        match cigar_op {
            // Match and SequenceMatch count as mapped bases
            rust_htslib::bam::record::Cigar::Match(len)
            | rust_htslib::bam::record::Cigar::Equal(len) => {
                current += len;
            }
            // Other operations break the contiguous stretch
            _ => {
                if current > longest {
                    longest = current;
                }
                current = 0;
            }
        }
    }

    // Check the last stretch
    if current > longest {
        longest = current;
    }

    longest
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate arguments
    if !(0.0..=1.0).contains(&args.complexity) {
        anyhow::bail!("Complexity cutoff must be between 0 and 1");
    }

    println!("Filtering paired-end BAM by kmer complexity and mapped bases");
    println!("  Input BAM: {}", args.input);
    println!("  Output BAM: {}", args.output);
    println!("  Complexity cutoff: {:.3}", args.complexity);
    if args.min_mapped > 0 {
        println!("  Min contiguous mapped bases: {} bp", args.min_mapped);
    }
    println!("  Kmer size: {}\n", KMER_SIZE);

    // Open input BAM file
    let mut bam_reader = bam::Reader::from_path(&args.input)?;

    // Get header
    let header = bam::Header::from_template(bam_reader.header());

    // Open output BAM file
    let mut bam_writer = bam::Writer::from_path(&args.output, &header, bam::Format::Bam)?;

    // Process pairs
    let mut total_pairs = 0u64;
    let mut filtered_pairs = 0u64;

    loop {
        // Read first record
        let mut record1 = bam::Record::new();
        match bam_reader.read(&mut record1) {
            Some(Ok(())) => {}
            None => break, // EOF
            Some(Err(e)) => {
                eprintln!("Error reading record: {}", e);
                break;
            }
        }

        // Read second record (mate)
        let mut record2 = bam::Record::new();
        match bam_reader.read(&mut record2) {
            Some(Ok(())) => {}
            None => {
                eprintln!("Warning: unpaired read at end of file");
                break;
            }
            Some(Err(e)) => {
                eprintln!("Error reading record: {}", e);
                break;
            }
        }

        total_pairs += 1;

        // Verify they're from the same pair (name-sorted)
        let name1 = std::str::from_utf8(record1.qname()).unwrap_or("");
        let name2 = std::str::from_utf8(record2.qname()).unwrap_or("");

        if name1 != name2 {
            anyhow::bail!(
                "BAM file not properly name-sorted!\n  Read 1: {}\n  Read 2: {}\n\
                 Please sort: samtools sort -n input.bam -o name_sorted.bam",
                name1, name2
            );
        }

        // Get sequences
        let seq1 = record1.seq().as_bytes();
        let seq2 = record2.seq().as_bytes();

        // Calculate complexity
        let complexity_r1 = calculate_kmer_complexity(&seq1);
        let complexity_r2 = calculate_kmer_complexity(&seq2);

        // Check mapped bases if filtering enabled
        let mapped_r1 = if args.min_mapped > 0 {
            get_longest_mapped_bases(&record1)
        } else {
            args.min_mapped
        };

        let mapped_r2 = if args.min_mapped > 0 {
            get_longest_mapped_bases(&record2)
        } else {
            args.min_mapped
        };

        // Filter: both reads must pass BOTH filters
        let pass_complexity =
            complexity_r1 >= args.complexity && complexity_r2 >= args.complexity;
        let pass_mapped = args.min_mapped == 0
            || (mapped_r1 >= args.min_mapped && mapped_r2 >= args.min_mapped);

        if pass_complexity && pass_mapped {
            bam_writer.write(&record1)?;
            bam_writer.write(&record2)?;
            filtered_pairs += 1;
        }

        // Progress report
        if total_pairs % 100000 == 0 {
            let pass_rate = if total_pairs > 0 {
                (filtered_pairs as f64 / total_pairs as f64) * 100.0
            } else {
                0.0
            };
            println!(
                "Processed {} pairs, kept {} ({:.1}%)",
                total_pairs, filtered_pairs, pass_rate
            );
        }
    }

    // Final report
    println!("\n=== Filtering Complete ===");
    println!("Total pairs: {}", total_pairs);
    println!("Filtered pairs: {}", filtered_pairs);
    println!("Removed pairs: {}", total_pairs - filtered_pairs);
    if total_pairs > 0 {
        let pass_rate = (filtered_pairs as f64 / total_pairs as f64) * 100.0;
        println!("Pass rate: {:.2}%", pass_rate);
    }
    println!("\nOutput file: {}", args.output);

    Ok(())
}
