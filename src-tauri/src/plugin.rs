use crate::error::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub struct PluginManager {
    plugins: Vec<Plugin>,
}

impl PluginManager {
    #[cfg(debug_assertions)]
    pub fn new() -> Self {
        let heading = LineFunction::new(
            "heading".to_string(),
            Some(r"^([#]).+".to_string()),
            vec![
                (
                    LineFunctionPositionArgument::Replace(
                        LineFunctionPosition::LineStart,
                        LineFunctionPosition::Index(0),
                    ),
                    "<h1>".to_string(),
                ),
                (
                    LineFunctionPositionArgument::Insert(LineFunctionPosition::Eol),
                    "</h1>".to_string(),
                ),
            ],
        )
        .expect("this works bro trust");

        let plugin = Plugin {
            name: "core".to_string(),
            line_functions: vec![heading],
        };

        Self {
            plugins: vec![plugin],
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Iterates over the the contained line functions and applies each one to the provided String.
    pub fn execute_line_functions(&mut self, input: &mut String) -> PluginExecutionResult<()> {
        let mut errors = Vec::new();
        for plugin in &mut self.plugins {
            // TODO: iterate over plugins and call the functions from the plugins themselves, instead of here.
            for function in &mut plugin.line_functions {
                if let Err(e) = function.apply_function(input) {
                    errors.extend(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Plugin {
    name: String,
    // fail_one_fail_all: bool,
    line_functions: Vec<LineFunction>,
}

/// These functions are contained within a line.
#[derive(Deserialize, Serialize)]
struct LineFunction {
    /// This is expected to be unique for each function.
    name: String,
    /// Every line that matches this regex pattern will be passed to the function as an individual argument.
    /// If the pattern is `None`, then every line is passed. This is not recommended as it can slow down the app for large inputs.
    pattern: Option<String>,
    /// The Regex is generated when the plugin is loaded, and then stored. It is reused whenever needed.
    /// This is where it is stored once generated.
    #[serde(skip)]
    #[serde(default = "return_none")]
    regex: Option<Regex>,
    /// This is the output of the function.
    /// `LineFunctionPositionArgument` states where to place the output.
    /// The `String` is the output itself.
    effect: Vec<(LineFunctionPositionArgument, String)>,
}

impl LineFunction {
    fn new(
        name: String,
        pattern: Option<String>,
        effect: Vec<(LineFunctionPositionArgument, String)>,
    ) -> PluginLoadResult<Self> {
        let regex = match pattern {
            None => {
                for (position, _) in &effect {
                    if position.has_index() {
                        return Err(PluginLoadError::IndexGivenWithoutPattern);
                    }
                }
                None
            }
            Some(ref pattern) => match regex::Regex::new(pattern) {
                Ok(r) => Some(r),
                Err(e) => {
                    return Err(PluginLoadError::InvalidRegex(format!(
                        "Regex engine returned an error: {}",
                        e
                    )));
                }
            },
        };

        Ok(Self {
            name,
            pattern,
            regex,
            effect,
        })
    }

    /// Tries to get regex if it exists already.
    /// The Regex is only generated once it has been called at least once.
    fn get_regex(&mut self) -> PluginExecutionResult<Option<&Regex>> {
        if let Some(ref re) = self.regex {
            // regex exists
            Ok(Some(re))
        } else if let Some(p) = &self.pattern {
            // a pattern exists, its regex doesn't, so we create it here
            self.regex = Some(
                Regex::new(p)
                    .map_err(|e| vec![FunctionExecutionError::InvalidRegex(format!("{}", e))])?,
            );
            Ok(self.regex.as_ref())
        } else {
            // neither exist, nothing needs to be changed
            Ok(None)
        }
    }

    fn apply_function(&mut self, input: &mut String) -> PluginExecutionResult<()> {
        let mut errors = Vec::new();
        let mut regex_index = None;

        if let Some(re) = self.get_regex()? {
            if let Some(m) = re.find(input) {
                regex_index = Some(m.start())
            } else {
                return Ok(());
            }
        }

        for (position, effect) in &self.effect {
            if let Err(e) = position.apply_effect_at_position(input, regex_index, effect) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Describes where the function effect should be placed.
#[derive(Deserialize, Serialize)]
enum LineFunctionPositionArgument {
    /// Index relative to regex result.
    Insert(LineFunctionPosition),
    /// Range relative to the regex result.
    Replace(LineFunctionPosition, LineFunctionPosition),
    /// Print a string to the log.
    /// The string that triggers this will also be printed to the log.
    /// Using this without a regex rule could flood the logs.
    Log(String),
    /// Print a string to the debugging log, not visible to the user by default.
    /// The string that triggers this will also be printed to the log.
    DebugLog(String),
}

impl LineFunctionPositionArgument {
    fn has_index(&self) -> bool {
        match self {
            LineFunctionPositionArgument::Insert(LineFunctionPosition::Index(_)) => true,
            LineFunctionPositionArgument::Replace(start, end) => {
                if let LineFunctionPosition::Index(_) = start {
                    return true;
                };
                if let LineFunctionPosition::Index(_) = end {
                    return true;
                };

                false
            }
            _ => false,
        }
    }

    fn apply_effect_at_position(
        &self,
        input: &mut String,
        regex_index: Option<usize>,
        effect: &str,
    ) -> PluginExecutionResult<()> {
        match self {
            LineFunctionPositionArgument::Insert(position) => {
                let i = position.get_index(input, regex_index)?;

                input.insert_str(i, effect);
                Ok(())
            }
            LineFunctionPositionArgument::Replace(start, end) => {
                let i = start.get_index(input, regex_index)?;
                let j = end.get_index(input, regex_index)?;

                input.replace_range(i..=j, effect);
                Ok(())
            }
            LineFunctionPositionArgument::Log(_message) => todo!(),
            LineFunctionPositionArgument::DebugLog(_message) => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize)]
enum LineFunctionPosition {
    /// Index with reference to the `start` property of the `Match` object returned by `Regex`.
    Index(isize),
    LineStart,
    /// End of Line
    Eol,
}

impl LineFunctionPosition {
    fn get_index(
        &self,
        input: &mut str,
        regex_index: Option<usize>,
    ) -> PluginExecutionResult<usize> {
        Ok(match self {
            LineFunctionPosition::Index(i) => {
                let index = match regex_index {
                    None => {
                        return Err(vec![FunctionExecutionError::EffectIndexGivenWithoutRegex]);
                    }
                    Some(regex_index) => regex_index as isize + i.to_owned(),
                };

                let input_length_isize: isize = input
                    .len()
                    .try_into()
                    .map_err(|_| vec![FunctionExecutionError::EffectIndexOutOfBounds])?;

                if (index < 0) || (index > input_length_isize) {
                    return Err(vec![FunctionExecutionError::EffectIndexOutOfBounds]);
                }

                index
                    .try_into()
                    .map_err(|_| vec![FunctionExecutionError::EffectIndexOutOfBounds])?
            }
            LineFunctionPosition::LineStart => 0,
            LineFunctionPosition::Eol => input.len(),
        })
    }
}

/// This is used for #[serde(default)] to initialise the regex property of functions.
fn return_none() -> Option<Regex> {
    None
}
