# Request Body common parameters between Completions and Chat API

| Name        | Completion | Chat | Both |
|-----------  |------------|------|------|
| model       | o          | o    |
| messages    | x          | o    |
| prompt      | o          | x    |
| suffix      | o          | x    |
| max_tokens  | o          | o    |
| temperature | o          | o    |
| top_p       | o          | o    |
| n           | o          | o    |
| stream      | o          | o    |
| logprobs    | o          | x    |
| echo        | o          | x    |
| stop        | o          | x    |
| presency_p  | o          | o    |
| frequency_p | o          | o    |
| best_of     | o          | x    |
| logit_bias  | o          | o    |
| user        | o          | o    |
