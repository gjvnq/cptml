# XML-NG

## Major Features

* The only named entities available are the ones in HTML. Ex: `&nbsp;` and `&forall;` ∀, `&sigmaf;` ς
* Long escaping with `<![[[ ` (the first space is mandatory and ignored) and `]]]>` (replaces `<!CDATA[[` and allows nesting by adding more brackets)
* Empty attribute just like in HTML. Ex: `<tag attr/>`
* No DTD, CDATA or other archaic non sense.
* Required heading `<?xml-ng?>`
* Special tags begin with colon `!` or `:` (avoids the need for long prefixes)
* Allow concurrent trees with layers prefixed with vertical pipe `|`
* Attributes can be repeated if they use brackets at the end, ex: `name[]="adjdj dj" name[]="ekdjdj"`
* Attributes don't require quotes if they are: numbers, `true` or `false`.
* Trailing whitespace is *always* ignored unless it is escaped (by ampersand or char data).

* Good APIs for rust and go (including canonization that minimizes space)
* API always tries to validate the document unless it is instructed not to or the URI is an empty string)

## Examples

## Whitespace

This:

```xml
<?xml-ng?>
<root>
	<tag/>
</root>
```

is the same as:

```xml
<?xml-ng?>
<root><tag/></root>
```

but not the same as:

```xml
<?xml-ng?>
<root>
&Tab;<tag/>
</root>
```

For simple spaces, use `&sp;`.

By the way, the valid "single letter" escapes are: `&a;`, `&b;`, `&f;`, `&n;`, `&r;`, `&t;`, `&v;`, `&&`, `&<`, `&>`.

### Import

```xml
<?xml-ng?>
<:import uri="other.xmlng" xpath="/cool-stuff">
	My <b>fallback</b> content
</:import>
```


By default the content is escaped if and only if it does not begin with a valid XML or XML-NG declaration.
Use the `escape` attribute to specify other behavior.

The DOM-NG will keep an indication where includes happen and a way to access the fallback content regardless of the success or failure in obtaining the desired resource.

### Base64

This is useful for singing stuff. The parser will auto decode the base64 content.

```xml
<?xml-ng?>
<:base64>
	SGVsbG8gPGk+V29ybGQ8L2k+IQ==
</:base64>
```

Will be "seen" by the DOM-NG as:

```xml
<?xml-ng?>
<:base64>
	Hello <i>World</i>!
</:base64>
```

The DOM-NG will provide an easy way to access the raw base64 content.

### Implicit endings

```
<?xml-ng?>
<document>
	<p>Hi,</>
	<p>Lorem ipsum ...
</document>
```

Is the same as

```
<?xml-ng?>
<document>
	<p>Hi,</p>
	<p>Lorem ipsum</p>
</document>
```

### Concurrent trees

Each concurrent tree is like its own separate documents which can contain multiple namespaces.

If a tag does not specify any concurrent tree, then it applies to all concurrent trees.

```xml
<?xml-ng?>
<!-- t| is for the typesetting tree -->
<!-- g| is for the grammar tree -->
<doc>
<page>
<t|line><g|sentence>I, by attorney, bless thee from thy mother,</t|line>
<t|line>Who prays continually for Richmond's good.</g|sentence></t|line>
</page>
<!-- Note it is possible to abbreviate the tag endings -->
<page>
<t|line><g|sentence>So much for that.</g|><g|sentence>—The silent hours steal on,</t|>
<t|line>And flaky darkness breaks within the east.</g|></t|>
</>
</doc>
```


When writing xpaths, the concurrent tree is specified in beginning. Example: `t|/page/line`, `g|//sentence`.

If a concurrent tree is not specified in an xpath, then only the default tree is considered.

Concurrent trees aren't limited to a single namespace. Example:

```xml-ng
<?xml-ng?>
<a|ns1:tag ns1:attr ns2:attr=10 />
<a|ns2:tag/>
```


TODO: see if there is a way to make this work as some sort of "semi-tree"

## DOM-NG 


