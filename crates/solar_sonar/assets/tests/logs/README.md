# Testing: Logs

EVE logs are written in UTF-16LE.

We maintain UTF8 as the source text for test log data.

We then convert the source text to UTF16 to be tested.

Any cherry-picking done from actual EVE logs needs to be converted and
scrubbed of user names, ids, and urls, etc. 

To convert between the two encodings, we use `iconv`.

## Convert from UTF-16LE to UTF-8:
`iconv -f utf-16le -t utf-8 read/from/log_file.txt > write/to/utf8_log_file.txt`

## Convert from UTF-8 to UTF-16LE:
`iconv -f utf-8 -t utf-16le read/from/utf8_log_file.txt > write/to/utf16_log_file.txt`
