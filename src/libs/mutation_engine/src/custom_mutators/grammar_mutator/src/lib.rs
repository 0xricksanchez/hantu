use errors::{Error, Result};
use prng::{Generator, Rng};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    path::PathBuf,
};

// Adapted from:
//  - <https://github.com/vrthra/F1>
//  - <https://github.com/gamozolabs/fzero_fuzzer>

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrammarTemplate {
    DataFormat(DataFormat),
    ImageFormat(ImageFormat),
    ArchiveFormat(ArchiveFormat),
    NetworkProtocol(NetworkProtocol),
    Multimedia(Multimedia),
    Document(Document),
    Font(Font),
    BinaryFormat(BinaryFormat),
    Book(Book),
    Custom(PathBuf),
}

impl From<String> for GrammarTemplate {
    fn from(name: String) -> Self {
        match name.as_str() {
            "json" => Self::DataFormat(DataFormat::Json),
            "html" => Self::DataFormat(DataFormat::Html),
            "xml" => Self::DataFormat(DataFormat::Xml),
            "csv" => Self::DataFormat(DataFormat::Csv),
            "ini" => Self::DataFormat(DataFormat::Ini),
            "yaml" => Self::DataFormat(DataFormat::Yaml),
            "sql_queries" => Self::DataFormat(DataFormat::SqlQueries),
            "jwt" => Self::DataFormat(DataFormat::Jwt),
            "markdown" => Self::DataFormat(DataFormat::Markdown),
            "geojson" => Self::DataFormat(DataFormat::GeoJson),
            "rtf" => Self::DataFormat(DataFormat::Rtf),
            "bson" => Self::DataFormat(DataFormat::Bson),
            "toml" => Self::DataFormat(DataFormat::Toml),
            "bash" => Self::DataFormat(DataFormat::Bash),
            "css" => Self::DataFormat(DataFormat::Css),
            "lua" => Self::DataFormat(DataFormat::Lua),
            "ruby" => Self::DataFormat(DataFormat::Ruby),
            "php" => Self::DataFormat(DataFormat::Php),
            "javascript" => Self::DataFormat(DataFormat::Javascript),
            "python" => Self::DataFormat(DataFormat::Python),
            "perl" => Self::DataFormat(DataFormat::Perl),
            "jpg" => Self::ImageFormat(ImageFormat::Jpg),
            "png" => Self::ImageFormat(ImageFormat::Png),
            "gif" => Self::ImageFormat(ImageFormat::Gif),
            "bmp" => Self::ImageFormat(ImageFormat::Bmp),
            "webp" => Self::ImageFormat(ImageFormat::WebP),
            "ico" => Self::ImageFormat(ImageFormat::Ico),
            "jpeg2000" => Self::ImageFormat(ImageFormat::Jpeg2000),
            "svg" => Self::ImageFormat(ImageFormat::Svg),
            "tiff" => Self::ImageFormat(ImageFormat::Tiff),
            "zip" => Self::ArchiveFormat(ArchiveFormat::Zip),
            "tar" => Self::ArchiveFormat(ArchiveFormat::Tar),
            "rar" => Self::ArchiveFormat(ArchiveFormat::Rar),
            "cpio" => Self::ArchiveFormat(ArchiveFormat::Cpio),
            "cab" => Self::ArchiveFormat(ArchiveFormat::Cab),
            "gzip" => Self::ArchiveFormat(ArchiveFormat::Gzip),
            "lzma" => Self::ArchiveFormat(ArchiveFormat::Lzma),
            "bzip2" => Self::ArchiveFormat(ArchiveFormat::Bzip2),
            "lzo" => Self::ArchiveFormat(ArchiveFormat::Lzo),
            "dns" => Self::NetworkProtocol(NetworkProtocol::Dns),
            "dhcp" => Self::NetworkProtocol(NetworkProtocol::Dhcp),
            "ntp" => Self::NetworkProtocol(NetworkProtocol::Ntp),
            "smtp" => Self::NetworkProtocol(NetworkProtocol::Smtp),
            "ftp" => Self::NetworkProtocol(NetworkProtocol::Ftp),
            "midi" => Self::Multimedia(Multimedia::Midi),
            "mp3" => Self::Multimedia(Multimedia::Mp3),
            "wav" => Self::Multimedia(Multimedia::Wav),
            "mp4" => Self::Multimedia(Multimedia::Mp4),
            "avi" => Self::Multimedia(Multimedia::Avi),
            "mov" => Self::Multimedia(Multimedia::Mov),
            "pdf" => Self::Document(Document::Pdf),
            "postscript" => Self::Document(Document::PostScript),
            "eps" => Self::Document(Document::Eps),
            "elf" => Self::BinaryFormat(BinaryFormat::Elf),
            "pe" => Self::BinaryFormat(BinaryFormat::Pe),
            "pcap" => Self::BinaryFormat(BinaryFormat::Pcap),
            "ebpf" => Self::BinaryFormat(BinaryFormat::Ebpf),
            "msgpack" => Self::BinaryFormat(BinaryFormat::MsgPack),
            "sqlitedb" => Self::BinaryFormat(BinaryFormat::SqliteDB),
            "ttf" => Self::Font(Font::Ttf),
            "woff" => Self::Font(Font::Woff),
            "epub" => Self::Book(Book::Epub),
            rem => Self::Custom(PathBuf::from(rem)),
        }
    }
}

impl GrammarTemplate {
    pub const NAMES: [&str; 62] = [
        "avi",
        "bash",
        "bmp",
        "bson",
        "bzip2",
        "cab",
        "cpio",
        "css",
        "csv",
        "dhcp",
        "dns",
        "ebpf",
        "elf",
        "eps",
        "epub",
        "ftp",
        "geojson",
        "gif",
        "gzip",
        "html",
        "ico",
        "ini",
        "javascript",
        "jpeg2000",
        "jpg",
        "json",
        "jwt",
        "lua",
        "lzma",
        "lzo",
        "markdown",
        "midi",
        "mov",
        "mp3",
        "mp4",
        "msgpack",
        "ntp",
        "pcap",
        "pdf",
        "pe",
        "perl",
        "php",
        "png",
        "postscript",
        "python",
        "rar",
        "rtf",
        "ruby",
        "smtp",
        "sql_queries",
        "sqlite_db",
        "svg",
        "tar",
        "tiff",
        "toml",
        "ttf",
        "wav",
        "webp",
        "woff",
        "xml",
        "yaml",
        "zip",
    ];

    fn get_path(&self) -> PathBuf {
        let base_path = Path::new(env!("CARGO_MANIFEST_DIR"));

        match self {
            Self::DataFormat(dformat) => match dformat {
                DataFormat::Json => base_path.join("grammars/json.json"),
                DataFormat::Html => base_path.join("grammars/html.json"),
                DataFormat::Xml => base_path.join("grammars/xml.json"),
                DataFormat::Csv => base_path.join("grammars/csv.json"),
                DataFormat::Ini => base_path.join("grammars/ini.json"),
                DataFormat::Yaml => base_path.join("grammars/yaml.json"),
                DataFormat::SqlQueries => base_path.join("grammars/sql_queries.json"),
                DataFormat::Jwt => base_path.join("grammars/jwt.json"),
                DataFormat::Markdown => base_path.join("grammars/markdown.json"),
                DataFormat::GeoJson => base_path.join("grammars/geojson.json"),
                DataFormat::Rtf => base_path.join("grammars/rtf.json"),
                DataFormat::Bson => base_path.join("grammars/bson.json"),
                DataFormat::Toml => base_path.join("grammars/toml.json"),
                DataFormat::Bash => base_path.join("grammars/bash.json"),
                DataFormat::Css => base_path.join("grammars/css.json"),
                DataFormat::Lua => base_path.join("grammars/lua.json"),
                DataFormat::Ruby => base_path.join("grammars/ruby.json"),
                DataFormat::Php => base_path.join("grammars/php.json"),
                DataFormat::Javascript => base_path.join("grammars/javascript.json"),
                DataFormat::Python => base_path.join("grammars/python.json"),
                DataFormat::Perl => base_path.join("grammars/perl.json"),
            },
            Self::ImageFormat(imgformat) => match imgformat {
                ImageFormat::Jpg => base_path.join("grammars/jpg.json"),
                ImageFormat::Png => base_path.join("grammars/png.json"),
                ImageFormat::Gif => base_path.join("grammars/gif.json"),
                ImageFormat::Bmp => base_path.join("grammars/bmp.json"),
                ImageFormat::WebP => base_path.join("grammars/webp.json"),
                ImageFormat::Ico => base_path.join("grammars/ico.json"),
                ImageFormat::Jpeg2000 => base_path.join("grammars/jpeg2000.json"),
                ImageFormat::Svg => base_path.join("grammars/svg.json"),
                ImageFormat::Tiff => base_path.join("grammars/tiff.json"),
            },
            Self::ArchiveFormat(arcformat) => match arcformat {
                ArchiveFormat::Zip => base_path.join("grammars/zip.json"),
                ArchiveFormat::Tar => base_path.join("grammars/tar.json"),
                ArchiveFormat::Rar => base_path.join("grammars/rar.json"),
                ArchiveFormat::Cpio => base_path.join("grammars/cpio.json"),
                ArchiveFormat::Cab => base_path.join("grammars/cab.json"),
                ArchiveFormat::Gzip => base_path.join("grammars/gzip.json"),
                ArchiveFormat::Lzma => base_path.join("grammars/lzma.json"),
                ArchiveFormat::Bzip2 => base_path.join("grammars/bzip2.json"),
                ArchiveFormat::Lzo => base_path.join("grammars/Lzo.json"),
            },
            Self::NetworkProtocol(netproto) => match netproto {
                NetworkProtocol::Dns => base_path.join("grammars/dns.json"),
                NetworkProtocol::Dhcp => base_path.join("grammars/dhcp.json"),
                NetworkProtocol::Ntp => base_path.join("grammars/ntp.json"),
                NetworkProtocol::Smtp => base_path.join("grammars/smtp.json"),
                NetworkProtocol::Ftp => base_path.join("grammars/ftp.json"),
            },
            Self::Multimedia(mformat) => match mformat {
                Multimedia::Midi => base_path.join("grammars/midi.json"),
                Multimedia::Mp3 => base_path.join("grammars/mp3.json"),
                Multimedia::Wav => base_path.join("grammars/wav.json"),
                Multimedia::Mp4 => base_path.join("grammars/mp4.json"),
                Multimedia::Avi => base_path.join("grammars/avi.json"),
                Multimedia::Mov => base_path.join("grammars/mov.json"),
            },
            Self::Document(docformat) => match docformat {
                Document::Pdf => base_path.join("grammars/pdf.json"),
                Document::PostScript => base_path.join("grammars/postscript.json"),
                Document::Eps => base_path.join("grammars/eps.json"),
            },
            Self::BinaryFormat(bformat) => match bformat {
                BinaryFormat::Elf => base_path.join("grammars/elf.json"),
                BinaryFormat::Pe => base_path.join("grammars/pe.json"),
                BinaryFormat::Pcap => base_path.join("grammars/pcap.json"),
                BinaryFormat::Ebpf => base_path.join("grammars/ebpf.json"),
                BinaryFormat::MsgPack => base_path.join("grammars/msgpack.json"),
                BinaryFormat::SqliteDB => base_path.join("grammars/sqlite_db.json"),
            },
            Self::Font(font) => match font {
                Font::Ttf => base_path.join("grammars/ttf.json"),
                Font::Woff => base_path.join("grammars/woff.json"),
            },
            Self::Book(book) => match book {
                Book::Epub => base_path.join("grammars/epub.json"),
            },
            Self::Custom(path) => {
                assert!(path.is_file(), "Grammar file not found: {path:?}");
                path.clone()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataFormat {
    Json,
    Html,
    Xml,
    Csv,
    Ini,
    Yaml,
    SqlQueries,
    Jwt,
    Toml,
    Markdown,
    GeoJson,
    Rtf,
    Bson,
    Bash,
    Css,
    Lua,
    Ruby,
    Php,
    Javascript,
    Python,
    Perl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Font {
    Ttf,
    Woff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Book {
    Epub,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    Jpg,
    Png,
    Gif,
    Bmp,
    WebP,
    Ico,
    Svg,
    Jpeg2000,
    Tiff,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    Rar,
    Cpio,
    Cab,
    Gzip,
    Lzma,
    Bzip2,
    Lzo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkProtocol {
    Dns,
    Dhcp,
    Ntp,
    Smtp,
    Ftp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Multimedia {
    Midi,
    Mp3,
    Wav,
    Mp4,
    Avi,
    Mov,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Document {
    Pdf,
    PostScript,
    Eps,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryFormat {
    SqliteDB,
    MsgPack,
    Pcap,
    Pe,
    Elf,
    Ebpf,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct TokenIdentifier(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // A list of tokens that should be expanded in order.
    OrderedExpansion(Vec<TokenIdentifier>),

    // A non-terminal token that should be expanded to a
    // random token from the given set.
    NonTerminal(Vec<TokenIdentifier>),

    // A terminal token that should be expanded to expanded
    // to the given bytes.
    Terminal(Vec<u8>),

    // Placeholder token for tokens that don't expand to anything.
    Nop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedJsonGrammar(BTreeMap<String, Vec<Vec<String>>>);

impl SerializedJsonGrammar {
    fn new<T: AsRef<Path> + ?Sized>(g: &T) -> Result<Self> {
        let g = std::fs::read_to_string(g)
            .map_err(|e| Error::new(&format!("Failed to read grammar from disk: {}", e)))?;
        let grammar: Self = serde_json::from_str(g.as_str())
            .map_err(|e| Error::new(&format!("Could not serialize grammar: {}", e)))?;
        Ok(grammar)
    }
}

impl Default for SerializedJsonGrammar {
    fn default() -> Self {
        let mut grammar = BTreeMap::new();
        grammar.insert("<start>".to_string(), vec![vec![]]);
        Self(grammar)
    }
}

#[derive(Debug, Default)]
pub struct Grammar {
    // The start token.
    pub start: Option<TokenIdentifier>,

    // A list of all tokens in the grammar.
    tokens: Vec<Token>,

    // A map from token names to token identifiers.
    token_map: BTreeMap<String, TokenIdentifier>,
}

impl PartialEq for Grammar {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.tokens == other.tokens
            && self.token_map == other.token_map
    }
}

impl Grammar {
    /// Creates a new Grammar instance from a SerializedJsonGrammar.
    ///
    /// The grammar is constructed in two stages:
    ///
    /// 1. Pre-populate the token list with non-terminal tokens, assigning a token
    ///    identifier to each non-terminal and adding it to the token map.
    /// 2. Create OrderedExpansion tokens for each non-terminal's possible expansions,
    ///    and update the non-terminal tokens with these expansions.
    ///
    /// Finally, the start token is set to the token identifier of the non-terminal token
    /// with the name "<start\>". If no such token exists, the start token is set to None.
    pub fn new(t: &GrammarTemplate) -> Result<Self> {
        let mut g = Self::default();
        let sjg = g.load_from_json(t)?;

        // Pre-populate the token list all non-terminal tokens.
        sjg.0.iter().for_each(|(non_term, _)| {
            assert!(g.token_map.get(non_term).is_none());
            let token_id = g.allocate_token(Token::NonTerminal(Vec::new()));
            g.token_map.insert(non_term.clone(), token_id);
        });

        // Construct the grammar.
        sjg.0.iter().for_each(|(non_term, values)| {
            let token_id = g.token_map[non_term];
            let mut ordered_exp = Vec::new();

            for val in values {
                let expansion_tokens = val
                    .iter()
                    .map(|token| {
                        if let Some(&non_term) = g.token_map.get(token) {
                            g.allocate_token(Token::NonTerminal(vec![non_term]))
                        } else {
                            g.allocate_token(Token::Terminal(token.as_bytes().to_vec()))
                        }
                    })
                    .collect::<Vec<_>>();

                let token_id = g.allocate_token(Token::OrderedExpansion(expansion_tokens));
                ordered_exp.push(token_id);
            }

            if let Token::NonTerminal(nt) = &mut g.tokens[token_id.0] {
                *nt = ordered_exp;
            }
        });

        // Resolve start node
        g.start = Some(g.token_map["<start>"]);

        // Return the constructed and optimized grammar.
        g.optimize();
        Ok(g)
    }

    fn load_from_json(&self, t: &GrammarTemplate) -> Result<SerializedJsonGrammar> {
        SerializedJsonGrammar::new(&t.get_path())
    }

    /// Allocates a new token in the grammar by appending it to the `tokens` vector and returning its identifier.
    ///
    /// # Arguments
    ///
    /// * `token` - A token instance to be added to the grammar.
    ///
    /// # Returns
    ///
    /// * `TokenIdentifier` - The identifier of the newly allocated token.
    pub fn allocate_token(&mut self, token: Token) -> TokenIdentifier {
        let token_id = TokenIdentifier(self.tokens.len());
        self.tokens.push(token);
        token_id
    }

    /// Optimize the grammar by removing tokens with non-random effects, such as `ordered_exp` with no actions or
    /// non-terminal tokens with only one option.
    ///
    /// This optimization step simplifies the grammar and makes it more efficient for generation purposes.
    fn optimize(&mut self) {
        let mut nop_tokens = BTreeSet::new();

        let mut changed = true;
        while changed {
            changed = false;
            for idx in 0..self.tokens.len() {
                match self.tokens[idx].clone() {
                    Token::NonTerminal(options) => {
                        if options.len() == 1 {
                            self.tokens[idx] = self.tokens[options[0].0].clone();
                            changed = true;
                        }
                    }
                    Token::OrderedExpansion(expansions) => {
                        if expansions.is_empty() {
                            self.tokens[idx] = Token::Nop;
                            changed = true;
                            nop_tokens.insert(idx);
                        }

                        if expansions.len() == 1 {
                            self.tokens[idx] = self.tokens[expansions[0].0].clone();
                            changed = true;
                        }

                        if let Token::OrderedExpansion(expansions) = &mut self.tokens[idx] {
                            expansions.retain(|x| {
                                if nop_tokens.contains(&x.0) {
                                    changed = true;
                                    false
                                } else {
                                    true
                                }
                            });
                        }
                    }
                    Token::Terminal(_) | Token::Nop => {}
                }
            }
        }
    }

    #[inline]
    fn get_token(&self, id: TokenIdentifier) -> &Token {
        &self.tokens[id.0]
    }

    /// Recursively generates a sequence of bytes from the grammar starting from the given token identifier.
    ///
    /// # Arguments
    ///
    /// * `depth` - The current depth of recursion.
    /// * `id` - The token identifier to start generating the sequence from.
    /// * `prng` - The mutable reference to the random number generator used for randomizing choices.
    /// * `out` - The mutable reference to the output byte vector where the generated sequence will be stored.
    ///
    /// # Notes
    ///
    /// The function limits the recursion depth to 128 to prevent infinite loops or stack overflows.
    pub fn generate(
        &self,
        depth: usize,
        id: TokenIdentifier,
        prng: &mut Rng<Generator>,
        out: &mut Vec<u8>,
    ) {
        // Limit recursion depth to prevent infinite loops or stack overflows
        // as specified in the original F1 paper
        if depth > 128 {
            return;
        }
        match self.get_token(id) {
            Token::Terminal(terminal) => {
                out.extend_from_slice(terminal);
            }
            Token::NonTerminal(options) => {
                let option = prng.pick(options);
                self.generate(depth + 1, *option, prng, out);
            }
            Token::OrderedExpansion(expansions) => {
                for expansion in expansions {
                    self.generate(depth + 1, *expansion, prng, out);
                }
            }
            Token::Nop => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prng::xorshift::Xorshift64;
    use prng::{Generator, Rng};
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn test_loading_json_grammars() {
        let entries = fs::read_dir("grammars").unwrap();
        for grammar in entries {
            let gpath = grammar.unwrap();
            if gpath.path().extension().unwrap() != "json" {
                continue;
            }
            let gpath = gpath.path();
            let grammar = SerializedJsonGrammar::new(&gpath);

            assert!(
                grammar.is_ok(),
                "Failed to load grammar: {:?} with error: {:?}",
                gpath,
                grammar
            );
        }
    }

    /// Create a dummy grammar for testing purposes.
    ///
    /// The grammar has the following structure:
    /// 0: NonTerminal -> [1]
    /// 1: OrderedExpansion -> [2, 3]
    /// 2: Terminal -> "A"
    /// 3: Terminal -> "B"
    /// 4: Nop
    /// 5: OrderedExpansion -> [2, 4, 3]
    ///
    /// Start token is TokenIdentifier(0)
    fn create_simple_dummy_grammar() -> Grammar {
        // Define the tokens for the dummy grammar
        let tokens = vec![
            Token::NonTerminal(vec![TokenIdentifier(1)]),
            Token::OrderedExpansion(vec![TokenIdentifier(2), TokenIdentifier(3)]),
            Token::Terminal(b"A".to_vec()),
            Token::Terminal(b"B".to_vec()),
            Token::Nop,
            Token::OrderedExpansion(vec![
                TokenIdentifier(2),
                TokenIdentifier(4),
                TokenIdentifier(3),
            ]),
        ];

        let mut token_map = BTreeMap::new();
        token_map.insert("<start>".to_string(), TokenIdentifier(0));

        Grammar {
            start: Some(TokenIdentifier(0)),
            tokens,
            token_map,
        }
    }

    #[test]
    /// Test the optimization function to make sure it optimizes the grammar correctly.
    ///
    /// Optimizations applied:
    /// 1. NonTerminal with a single option is replaced by the option itself.
    ///    - Token 0 (NonTerminal) will be replaced by Token 1 (OrderedExpansion).
    ///
    /// No other optimizations are applied because:
    /// - All other tokens are either Terminal or Nop, which are already maximally optimized.
    /// - The Nop token in Token 5 (OrderedExpansion) is not removed because it's not the only element.
    fn test_simple_optimization() {
        let mut grammar = create_simple_dummy_grammar();
        grammar.optimize();

        // Expected optimized grammar
        let optimized_tokens = vec![
            Token::OrderedExpansion(vec![TokenIdentifier(2), TokenIdentifier(3)]),
            Token::OrderedExpansion(vec![TokenIdentifier(2), TokenIdentifier(3)]),
            Token::Terminal(b"A".to_vec()),
            Token::Terminal(b"B".to_vec()),
            Token::Nop,
            Token::OrderedExpansion(vec![
                TokenIdentifier(2),
                TokenIdentifier(4),
                TokenIdentifier(3),
            ]),
        ];

        let mut optimized_token_map = BTreeMap::new();
        optimized_token_map.insert("<start>".to_string(), TokenIdentifier(0));

        let expected_optimized_grammar = Grammar {
            start: Some(TokenIdentifier(0)),
            tokens: optimized_tokens,
            token_map: optimized_token_map,
        };

        assert_eq!(grammar, expected_optimized_grammar);
    }

    /// Creates a complex dummy grammar used for testing.
    ///
    /// This function generates a `Grammar` object that represents a non-optimized
    /// complex grammar, containing various `Token`s such as `NonTerminal`,
    /// `OrderedExpansion`, `Terminal`, and `Nop`.
    ///
    /// Returns the generated `Grammar` object.
    fn create_complex_dummy_grammar() -> Grammar {
        let tokens = vec![
            Token::NonTerminal(vec![TokenIdentifier(1)]),
            Token::OrderedExpansion(vec![TokenIdentifier(2), TokenIdentifier(3)]),
            Token::Terminal(b"A".to_vec()),
            Token::Terminal(b"B".to_vec()),
            Token::NonTerminal(vec![TokenIdentifier(5)]),
            Token::OrderedExpansion(vec![TokenIdentifier(6), TokenIdentifier(7)]),
            Token::Terminal(b"C".to_vec()),
            Token::Terminal(b"D".to_vec()),
            Token::Nop,
            Token::OrderedExpansion(vec![TokenIdentifier(8)]),
            Token::OrderedExpansion(vec![
                TokenIdentifier(2),
                TokenIdentifier(8),
                TokenIdentifier(3),
            ]),
        ];

        let token_map = vec![("<start>".to_string(), TokenIdentifier(0))]
            .into_iter()
            .collect::<BTreeMap<String, TokenIdentifier>>();

        Grammar {
            start: Some(TokenIdentifier(0)),
            tokens,
            token_map,
        }
    }

    /// Creates an optimized version of the complex dummy grammar used for testing.
    ///
    /// This function generates a `Grammar` object that represents an optimized
    /// version of the complex dummy grammar created by `create_complex_dummy_grammar`.
    /// This optimized grammar has some redundant tokens removed or simplified.
    ///
    /// Returns the generated optimized `Grammar` object.
    fn create_optimized_complex_dummy_grammar() -> Grammar {
        let tokens = vec![
            Token::OrderedExpansion(vec![TokenIdentifier(2), TokenIdentifier(3)]),
            Token::OrderedExpansion(vec![TokenIdentifier(2), TokenIdentifier(3)]),
            Token::Terminal(b"A".to_vec()),
            Token::Terminal(b"B".to_vec()),
            Token::OrderedExpansion(vec![TokenIdentifier(6), TokenIdentifier(7)]),
            Token::OrderedExpansion(vec![TokenIdentifier(6), TokenIdentifier(7)]),
            Token::Terminal(b"C".to_vec()),
            Token::Terminal(b"D".to_vec()),
            Token::Nop,
            Token::Nop,
            Token::OrderedExpansion(vec![
                TokenIdentifier(2),
                TokenIdentifier(8),
                TokenIdentifier(3),
            ]),
        ];

        let token_map = vec![("<start>".to_string(), TokenIdentifier(0))]
            .into_iter()
            .collect::<BTreeMap<String, TokenIdentifier>>();

        Grammar {
            start: Some(TokenIdentifier(0)),
            tokens,
            token_map,
        }
    }

    #[test]
    fn test_complex_optimization() {
        let mut complex_grammar = create_complex_dummy_grammar();
        complex_grammar.optimize();

        let expected_grammar = create_optimized_complex_dummy_grammar();

        assert_eq!(complex_grammar, expected_grammar);
    }

    #[test]
    fn generate_complex_dummy_grammar() {
        let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0xdeadbeefcafebabe)));
        let mut grammar = create_complex_dummy_grammar();

        grammar.optimize();
        let mut res = Vec::new();
        grammar.generate(0, grammar.start.unwrap(), &mut prng, &mut res);
        assert_eq!(res, b"AB");
    }

    #[test]
    fn generate_larger_json() {
        let mut prng = Rng::new(Generator::Xorshift64(Xorshift64::new(0)));
        let grammar = Grammar::new(&GrammarTemplate::DataFormat(DataFormat::Json)).unwrap();

        let mut res = Vec::new();
        while res.len() < 500 {
            res.clear();
            grammar.generate(0, grammar.start.unwrap(), &mut prng, &mut res);
        }
        //println!("{}", String::from_utf8_lossy(&res));
        //fs::write("test.yml", &res).unwrap();
        assert!(res.len() >= 500);
    }
}
