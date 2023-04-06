use custom_error::custom_error;

custom_error! {pub OpenAi
    NoChoices = "no chat choices",
    InvalidStop{stop: String} = "stop value ({stop}) must be either 'left' or 'right'",
    RequestError{body: String} = "request error: {body}",
    ModelNotFound{model_name: String} = "model not found: {model_name}",
    SerializationError{body: String} = "serialization error: {body}",
    ClientError{body: String} = "client error:\n{body}",
    InvalidLogProbs{logprobs: f32} = "logprob value ({logprobs}) must be between 0 and 5",
    InvalidEcho = "'echo' cannot be used with 'suffix'",
    InvalidStream = "'stream' cannot be used with 'best_of'",
    InvalidSuffix = "'suffix' cannot be used with 'echo'",
    InvalidBestOf = "'best_of' cannot be used with 'stream'",
    InvalidTemperature{temperature: f32} = "temperature value ({temperature}) must be between 0.0 and 2.0",
    InvalidTopP{top_p: f32} = "top_p value ({top_p}) must be between 0 and 1",
    InvalidPresencePenalty{presence_penalty: f32} = "presence_penalty value ({presence_penalty}) must be between -2.0 and 2.0",
    InvalidFrequencyPenalty{frequency_penalty: f32} = "frequency_penalty ({frequency_penalty}) must be between -2.0 and 2.0",
}
