pub type PluginLoadResult<T> = Result<T, PluginLoadError>;

#[derive(Debug)]
pub enum PluginLoadError {
    /// There is no regex pattern given to get an index from, but there's an effect that depends on an index.
    IndexGivenWithoutPattern,
    /// Provided regex was invalid.
    InvalidRegex(String),
}

pub type PluginExecutionResult<T> = Result<T, Vec<FunctionExecutionError>>;

#[derive(Debug)]
pub enum FunctionExecutionError {
    /// Provided regex was invalid.
    InvalidRegex(String),
    /// The effect index must lie within the line. It can't be less than 0 or more than its length.
    EffectIndexOutOfBounds,
    /// The effect index is given without using a regex. This function should not have been loaded.
    EffectIndexGivenWithoutRegex,
}
