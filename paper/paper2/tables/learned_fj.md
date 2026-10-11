| universe | model | k | best other by J' | J' learned - best [95% CI] | best other by C + FJ | C + FJ learned - best [95% CI] |
| --- | --- | ---: | --- | ---: | --- | ---: |
| 3,000 books | minilm | 10 | `lcc-gold` | -0.016 [-0.043, +0.007] | `lcc-gold` | +0.140 [+0.111, +0.167] |
| 3,000 books | minilm | 20 | `ddc-section` | -0.105 [-0.129, -0.083] | `lcc-full-gold` | +0.045 [+0.020, +0.068] |
| 3,000 books | minilm | 50 | `ddc-d2` | -0.152 [-0.171, -0.134] | `lcc-gold` | -0.037 [-0.059, -0.016] |
| 3,000 books | bge-small | 10 | `lcc-gold` | +0.018 [-0.008, +0.040] | `lcc-gold` | +0.183 [+0.155, +0.209] |
| 3,000 books | bge-small | 20 | `ddc-section` | -0.052 [-0.075, -0.032] | `lcc-full-gold` | +0.092 [+0.068, +0.115] |
| 3,000 books | bge-small | 50 | `ddc-d2` | -0.097 [-0.116, -0.079] | `ddc-gold` | +0.015 [-0.007, +0.038] |
| 3,000 books | nomic | 10 | `lcc-gold` | +0.036 [+0.010, +0.060] | `lcc-gold` | +0.195 [+0.168, +0.223] |
| 3,000 books | nomic | 20 | `ddc-section` | -0.058 [-0.082, -0.036] | `lcc-full-gold` | +0.098 [+0.072, +0.123] |
| 3,000 books | nomic | 50 | `ddc-section` | -0.108 [-0.129, -0.090] | `lcc-gold` | +0.010 [-0.012, +0.033] |
| 10,000 papers (in sample) | minilm | 20 | `fdc-arxiv-nested` | -0.136 [-0.145, -0.128] | `arxiv-gold` | -0.041 [-0.051, -0.031] |
| 10,000 papers (in sample) | bge-small | 20 | `fdc-arxiv-nested` | -0.115 [-0.122, -0.106] | `arxiv-gold` | -0.046 [-0.056, -0.036] |
| 10,000 papers (in sample) | nomic | 20 | `fdc-arxiv-nested` | -0.116 [-0.124, -0.108] | `arxiv-gold` | -0.036 [-0.047, -0.027] |
| 500 holdout books (title only) | minilm | 20 | `lcc-pipeline` | -0.264 [-0.309, -0.221] | `lcc-full-gold` | -0.249 [-0.306, -0.192] |
| 500 holdout books (title only) | bge-small | 20 | `fdc-pipeline` | -0.181 [-0.226, -0.137] | `fdc-work` | -0.110 [-0.168, -0.053] |
| 500 holdout books (title only) | nomic | 20 | `fdc-work` | -0.184 [-0.235, -0.126] | `fdc-work` | -0.134 [-0.198, -0.066] |
