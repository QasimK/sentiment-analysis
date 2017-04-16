# Sentiment Analysis

A simple project to learn Rust that will

1. Download the specified web-page / use the text directly given it to
2. Output a sentiment value from -100 to 100 (where 100 is maximum positivity)

It works with some web-pages better than others because the exact article
can be selected from the page. See web_selectors.txt.

## Usage

    sentiment https://example.org/article/
    sentiment "This is some happy, glorious text that is amazing."
    sentiment --help
