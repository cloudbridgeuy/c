use color_eyre::eyre::Result;
use crossterm::terminal;

// Markdown language constant string
const THEME: &str = "tokyonight-storm";
const LANGUAGE: &str = "markdown";

pub struct CustomPrinter<'a> {
    inputs: Vec<bat::input::Input<'a>>,
    config: bat::config::Config<'a>,
    assets: bat::assets::HighlightingAssets,
    term_width: Option<usize>,
}

impl<'a> CustomPrinter<'a> {
    pub fn new() -> Self {
        let config = bat::config::Config {
            colored_output: true,
            true_color: true,
            language: Some(LANGUAGE),
            theme: THEME.to_string(),
            use_italic_text: true,
            wrapping_mode: bat::WrappingMode::Character,
            ..Default::default()
        };

        CustomPrinter {
            inputs: vec![],
            config,
            assets: bat::assets::HighlightingAssets::from_binary(),
            term_width: None,
        }
    }

    /// Add a byte string as an input
    pub fn input_from_bytes(&mut self, content: &'a [u8]) -> &mut Self {
        self.input_from_reader(content)
    }

    /// Add a custom reader as an input
    pub fn input_from_reader<R: std::io::Read + 'a>(&mut self, reader: R) -> &mut Self {
        self.inputs
            .push(bat::input::Input::from_reader(Box::new(reader)));
        self
    }

    /// Custom print function that takes advantage of the fact that `bat` controllers can take a
    /// String as the output of the highlighted text.
    pub fn print(&mut self) -> Result<String> {
        self.config.term_width = self
            .term_width
            .unwrap_or_else(|| terminal::size().unwrap().0 as usize);
        let inputs = std::mem::take(&mut self.inputs);

        let mut output = String::new();

        let controller = bat::controller::Controller::new(&self.config, &self.assets);
        controller.run(inputs, Some(&mut output))?;

        Ok(output)
    }
}
