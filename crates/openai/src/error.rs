use custom_error::custom_error;

custom_error! {pub OpenAiError
    InvalidTemperature{temperature: f32} = "temperature value ({temperature}) must be between 0 and 2",
    InvalidTopP{top_p: f32} = "top_p value ({top_p}) must be between 0 and 1",
    InvalidPresencePenalty{presence_penalty: f32} = "presence_penalty value ({presence_penalty}) must be between -2.0 and 2.0",
    InvalidFrequencyPenalty{frequency_penalty: f32} = "frequency_penalty ({frequency_penalty}) must be between -2.0 and 2.0",
}
