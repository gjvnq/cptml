# CPTML

CPTML = Curly & Pointy Tags Markup Language

(also joke with CPTM: *Companhia Paulista de Trens Metropolitanos*)

## Main features

  * Compact syntax: ```{tag attr="value"; inner text}```
  * Namespace support: ```{ns1.tag ns2.attr="value"; inner text}```
  * "Empty" namespace for special things like id and language: ```{a .id="spanish-link" .lang="es-419" .href=<example.com/es>}```
    * Roughly equivalent to ```xml:id``` and ```xml:lang```
    * Using `.href` means that the tag will be clickable in the official CPTML viewer.
  * Typed attributes (string, int, float, boolean, URI/IRI and list). ```{tag .id="section1 str-attr="my string!" int-attr=123 float-attr=1.12345E7 bool-attr=true uri-attr=<ftp://example.com/page#id> uri-mail-attr=<mailto:user@example.com> local-uri-attr=<#section2> class=["title" "bold"]}```
    * URI attributtes will be clickable in the official CPTML viewer.
    * URI attributes assume HTTP(S) if no protocol is specified.
  * Native support for including other files.
  * Overlapping markup support via multiple trees/views.
  * Text and comments show as special elements on the tree/API.
  * C-style escape sequences. (no XML entities nonsense)
  * Nice syntax for escaped text: ````$$$```` (for code blocks), ```$$``` (for LaTeX display math) and ```$``` (for LaTeX inline math).
  * Can include the language name in code blocks: ```$$$rust```
  * First character of escaped text is ignored if it is a space (U+0020). Example: ```$ $``` produces nothing, ```$ $$``` produces ```$```, ```$  $$``` produces ``` $```. (This is useful for escaping LaTeX code)

## Examples

### US Constitution (Article I Section I)

```cptml
{preamble .id="preamble";
  {recital .id="s1"; {span class="small-caps"; We the People} of the United States, in Order to form a more perfect Union, establish Justice, insure domestic Tranquility, provide for the common defence, promote the general Welfare, and secure the Blessings of Liberty to ourselves and our Posterity, do ordain and establish this Constitution for the United States of America.}}
{article .id="article-I" num=1;
  {heading; ARTICLE I.}
  {section .id="article-I-1" num=1;
    {heading; Section 1.}
    {paragraph .id="s3"; All legislative Powers herein granted shall be vested in a Congress of the United States, which shall consist of a Senate and House of Representatives.}
}}
```

![](constitution.png)

Text source: https://github.com/usgpo/house-manual/blob/master/114/original-file-names/constitution.xml

### Overlapping Markup

This document has three views: the empty/default one, `t` for typography and `g` for grammar.

Note that ending tags can be abbreviated: `|>` (default namespace) and `|/t>` (for view `t`).

```cptml
{poem;
  <t/line|<g/sentence|I, by attorney, bless thee from thy mother,|t/line>
  <t/line|Who prays continually for Richmond's good.|g/sentence>|t/line>
  <t/line|<g/sentence|So much for that.|/g><g/sentence|—The silent hours steal on,|t/>
  <t/line|And flaky darkness breaks within the east.|g/>|t/>
}
```

![](poem.png)

Text source: https://en.wikipedia.org/wiki/Overlapping_markup#Milestones

### Math

```cptml
{p; The quadratic formula is below:}

$$\frac{-b\pm\sqrt{b^2-4ac}}{2a}$$
```

## Escape sequences

  * `\a`: Alert or bell
  * `\b`: Backspace
  * `\\`: Backslash
  * `\s`: Whitespace
  * `\t`: Horizontal tab
  * `\n`: Line feed or newline
  * `\f`: Form feed
  * `\r`: Carriage return
  * `\uN...N;`: Unicode character code point (where `N...N` is one or more hexadecimal digits)
    * Surrogates are not allowed
  * `\v`: Vertical tab
  * `\'`: Single quote
  * `\"`: Double quote
  * `\{`: Begin curly brace
  * `\}`: End curly brace
  * `\<`: Less than
  * `\>`: Greate than
  * `\|`: Vertical pipe
  * <code>\`</code>: Back tick

## White space relevance

  * Whitespace = blank space (0x20) and tab (0x09) not including escaped versions of them (ex: `\s` and `\t`).
  * Whitespace is ignored:
    * From the begining of a line until first non-whitespace char.
    * From the last non-whitespace char until the end of the line.

## Special Elements

### `.cptml`

Indicated the file type and version.

### `.schema`

Specifies where the schema is locate and defines namespaces.

Attributes:

  * `ns` (required): string with the namespace prefix. (is empty for the default/main namespace)
  * `href` (optional): the URL where the schema is available.
  * `nsid` (optional): the unique identifier of this namespace.

At least one of `!href` and `uid` must be present.

### `.root`

Exists just to make sure all documents have a non empty root. This is never transcribed to output.

### `.include`

Indicates that another file is to be included. By default, it is included as if it were properly escaped text.

Attributes:

  * `src` (required): string with a file URI.
  * `parse` (optional): if `true`, will parse the referenced file as CPTML.

Note: by default `.include` nodes won't appear on the tree and won't be considered when processing tree paths.

### `.text`

Represents a text node. It is essentially a "virtual element" used to simplify the APIs and canonize documents.

Attributes:

  * `val`: the textual data as a string including only the relevant whitespace.
  * `fencing`: number of $ (dollar signs) used in the text

### `.whitespace`

Respresents irrelevant whitespace, basically a "virtual" element to simplify the API.

Attributes:

  * `val`: the whitespace deemed irrelevant as a string.

## Special Attributes

Any regular elements may have the following attributes which work across namespaces.

  * `.id`: equivalent of `xml:id`
  * `.lang`: equivalent of `xml:lang`
  * `.href`: makes the element clickable (value must be an IRI)

## Tree Paths

Very similar to XPath but simpler.

The following selectors are available:

* `.`: context node.
* `..`: parent of the context node.
* `*`: all children of the context node (except `.text`).
* `node()`, `**`: all children of the context node including text.
* `text()`: all text children of the context node (includes escaped text).
* `inner-text()`: a single string with all the text under the context node.
* `tag`: all child elements with a name matching the `tag`.
* `@name`: attribute `name` of the context node.
* `@*`: all attributes of the context node.
* `/`: root element when used at the start of a path.
* `///`: all descendants of the root.
* `//`: all descendants of the context node.

The following basic filters are supported:

* `[@attrib]`: keep elements with an attribute named `attrib`.
* `[@attrib='val']`: keep elements with an attribute named `attrib` and value matching `val`.
* `[tag]`           keep elements with a child element named `tag`.
* `[tag='val']`     keep elements with a child element named `tag` and text matching `val`.
* `[n]`: keep the `n`-th element, where `n` is a numeric index starting from 1.

The following function-based filters are supported:

* `[text()]`: keep elements with non-empty text.
* `[text()='val']`: keep elements whose text matches `val`.

The following set operators are supported:

* ` + `: union of two sets (like ` | ` in XPath)
* ` - `: subtraction of two sets
* ` & `: intersection of two sets

The view is specified in the beginning of the path and separated with a vertical pipe. Ex: `v|/root/child`

The search is always depth-first.

## Planned Typesetting features

Basic ones: paragraph, bold, italic, striked, underlined, overlined, color, quote, blockquote, code, image.

Intermediary ones:
  * Tables
  * Multiple readings (like TEI's `<choice>`)
  * Interlinear annotations (mainly for ruby)
  * Leipzig annotations
  * Diff annotations
  * Leiden conventions
  * Academic citations
  * Parallel texts
  * Multiple columns
  
Advanced ones:
  * User toggable details (e.g. a button to hide all IPA transcriptions, hide all translations from Spanish, show all term definitions)
  * Glossary with tooltips
  * Footnotes with tooltips
  * Treebanks
  * Diagrams
  * Translation equivalence by hovering over the terms (just like on Google Translate)
