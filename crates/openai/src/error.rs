use custom_error::custom_error;

custom_error! {pub OpenAi
    ClientError{body: String} = "client error:\n{body}",
    DeserializationError{body: String} = "deserialization error: {body}",
    FileError{body: String} = "file error: {body}",
    InvalidBestOf = "'best_of' cannot be used with 'stream'",
    InvalidEcho = "'echo' cannot be used with 'suffix'",
    InvalidFrequencyPenalty{frequency_penalty: f32} = "frequency_penalty ({frequency_penalty}) must be between -2.0 and 2.0",
    InvalidLogProbs{logprobs: f32} = "logprob value ({logprobs}) must be between 0 and 5",
    InvalidPresencePenalty{presence_penalty: f32} = "presence_penalty value ({presence_penalty}) must be between -2.0 and 2.0",
    InvalidStop{stop: String} = "stop value ({stop}) must be either 'left' or 'right'",
    InvalidStream = "'stream' cannot be used with 'best_of'",
    InvalidSuffix = "'suffix' cannot be used with 'echo'",
    InvalidTemperature{temperature: f32} = "temperature value ({temperature}) must be between 0.0 and 2.0",
    InvalidTopP{top_p: f32} = "top_p value ({top_p}) must be between 0 and 1",
    ModelNotFound{model_name: String} = "model not found: {model_name}",
    NoChoices = "no chat choices",
    NoSession = "no session",
    RequestError{body: String} = "request error: {body}",
    SerializationError{body: String} = "serialization error: {body}",
    TrimError = "could not find a message to trim",
    UknownError = "unknown error",
}
