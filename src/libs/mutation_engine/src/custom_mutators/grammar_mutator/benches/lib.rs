#![feature(test)]

extern crate grammar_mutator;
extern crate prng;
extern crate test;

use grammar_mutator::{
    ArchiveFormat, BinaryFormat, Book, DataFormat, Document, Font, Grammar, GrammarTemplate,
    ImageFormat, Multimedia, NetworkProtocol,
};
use prng::xorshift::Xorshift64;
use prng::{Generator, Rng};
use std::time::Instant;
use test::{black_box, Bencher};

const ITERATIONS: usize = 100;

fn all_grammar_templates() -> Vec<GrammarTemplate> {
    vec![
        GrammarTemplate::DataFormat(DataFormat::Json),
        GrammarTemplate::DataFormat(DataFormat::Html),
        GrammarTemplate::DataFormat(DataFormat::Xml),
        GrammarTemplate::DataFormat(DataFormat::Csv),
        GrammarTemplate::DataFormat(DataFormat::Ini),
        GrammarTemplate::DataFormat(DataFormat::Yaml),
        GrammarTemplate::DataFormat(DataFormat::SqlQueries),
        GrammarTemplate::DataFormat(DataFormat::Jwt),
        GrammarTemplate::DataFormat(DataFormat::Toml),
        GrammarTemplate::DataFormat(DataFormat::Markdown),
        GrammarTemplate::DataFormat(DataFormat::GeoJson),
        GrammarTemplate::DataFormat(DataFormat::Rtf),
        GrammarTemplate::DataFormat(DataFormat::Bson),
        GrammarTemplate::DataFormat(DataFormat::Bash),
        GrammarTemplate::DataFormat(DataFormat::Css),
        GrammarTemplate::DataFormat(DataFormat::Lua),
        GrammarTemplate::DataFormat(DataFormat::Ruby),
        GrammarTemplate::DataFormat(DataFormat::Php),
        GrammarTemplate::DataFormat(DataFormat::Javascript),
        GrammarTemplate::DataFormat(DataFormat::Python),
        GrammarTemplate::DataFormat(DataFormat::Perl),
        GrammarTemplate::Font(Font::Ttf),
        GrammarTemplate::Font(Font::Woff),
        GrammarTemplate::ImageFormat(ImageFormat::Jpg),
        GrammarTemplate::ImageFormat(ImageFormat::Png),
        GrammarTemplate::ImageFormat(ImageFormat::Gif),
        GrammarTemplate::ImageFormat(ImageFormat::Bmp),
        GrammarTemplate::ImageFormat(ImageFormat::WebP),
        GrammarTemplate::ImageFormat(ImageFormat::Ico),
        GrammarTemplate::ImageFormat(ImageFormat::Svg),
        GrammarTemplate::ImageFormat(ImageFormat::Jpeg2000),
        GrammarTemplate::ImageFormat(ImageFormat::Tiff),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Zip),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Tar),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Rar),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Cpio),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Cab),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Gzip),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Lzma),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Bzip2),
        GrammarTemplate::ArchiveFormat(ArchiveFormat::Lzo),
        GrammarTemplate::NetworkProtocol(NetworkProtocol::Dns),
        GrammarTemplate::NetworkProtocol(NetworkProtocol::Dhcp),
        GrammarTemplate::NetworkProtocol(NetworkProtocol::Ntp),
        GrammarTemplate::NetworkProtocol(NetworkProtocol::Smtp),
        GrammarTemplate::NetworkProtocol(NetworkProtocol::Ftp),
        GrammarTemplate::Multimedia(Multimedia::Midi),
        GrammarTemplate::Multimedia(Multimedia::Mp3),
        GrammarTemplate::Multimedia(Multimedia::Wav),
        GrammarTemplate::Multimedia(Multimedia::Mp4),
        GrammarTemplate::Multimedia(Multimedia::Avi),
        GrammarTemplate::Multimedia(Multimedia::Mov),
        GrammarTemplate::Document(Document::Pdf),
        GrammarTemplate::Document(Document::PostScript),
        GrammarTemplate::Document(Document::Eps),
        GrammarTemplate::BinaryFormat(BinaryFormat::SqliteDB),
        GrammarTemplate::BinaryFormat(BinaryFormat::MsgPack),
        GrammarTemplate::BinaryFormat(BinaryFormat::Pcap),
        GrammarTemplate::BinaryFormat(BinaryFormat::Pe),
        GrammarTemplate::BinaryFormat(BinaryFormat::Elf),
        GrammarTemplate::BinaryFormat(BinaryFormat::Ebpf),
        GrammarTemplate::Book(Book::Epub),
    ]
}

fn bench_fn(
    grammar: &Grammar,
    prng: &mut Rng<Generator>,
    res: &mut Vec<u8>,
    generated: &mut usize,
) {
    for _ in 0..ITERATIONS {
        res.clear();
        grammar.generate(0, grammar.start.unwrap(), prng, res);

        *generated += res.len();
    }
}

#[bench]
fn bench_all_grammars(b: &mut Bencher) {
    let grammar_templates = all_grammar_templates();
    for template in grammar_templates {
        let grammar = Grammar::new(&template).unwrap();
        let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));

        let mut generated = 0;
        let mut res = Vec::new();

        let start = Instant::now();
        b.iter(|| {
            let it = Instant::now();
            bench_fn(&grammar, &mut prng, &mut res, &mut generated);
            let elapsed = (Instant::now() - it).as_secs_f64();
            black_box(elapsed); // Prevent the compiler from optimizing away the code
        });
        let total_elapsed = (Instant::now() - start).as_secs_f64();

        let bytes_per_sec = generated as f64 / total_elapsed;
        println!(
            "Grammar: {:?}, MiB/s: {:>12.2}",
            template,
            bytes_per_sec / 1024.0 / 1024.0
        );
    }
}
