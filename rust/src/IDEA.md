* Use a two passes parser:
  * 1st pass: parse the syntax tree (in a one to one correspondence)
  * 2nd pass: process the text nodes (whitespace relevance and character escape codes) and the language tags on code blocks. (this process cannot be unambiguosly reverted)

* I discovered that nom suports nice verbose errors, so I may use it. (see https://github.com/Geal/nom/blob/master/examples/json.rs#L300)