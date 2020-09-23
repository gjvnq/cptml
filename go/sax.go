package tlnml

import (
	"bufio"
	"errors"
	"io"
	"regexp"
	"strconv"
	"strings"
	"unicode"
)

const TEX_STYLE = '\''
const XML_STYLE = '<'
const PROC_INST_STYLE = '?'

/// Simple attribute values that can be incorporated without quotation marks
var RE_NOQUOTE = regexp.MustCompile(`^[\p{L}\p{N}_-]*`)
var RE_NAME_START = regexp.MustCompile(`[\p{Ll}\p{Lt}\p{Lu}\p{Lo}]`) // Note the abscence of \p{Lm}
var RE_NAME_CONT = regexp.MustCompile(`[\p{L}\p{N}\p{S}]*`)

var NonParsedCharData = errors.New("CharData was not parsed")

type NameOrString interface{}

type Comment string
type CharData struct {
	/// Number of brackets used/necessary to escape the text. Zero if "normal" inline text.
	bracketsNum int
	raw         strings.Builder
	parsed      strings.Builder
	isParsed    bool
	mode        int
}

func (dat CharData) String() string {
	if !dat.isParsed {
		panic(NonParsedCharData)
	}
	return dat.parsed.String()
}

func (dat *CharData) parse() error {
	if dat.isParsed {
		return nil
	}

	input := strings.NewReader(dat.raw.String())
	dat.parsed.Reset()
	holdStr := strings.Builder{}
	holdRune := '�'
	for {
		r, _, err := input.ReadRune()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}
		if dat.mode == 0 {
			// mode: Regular text
			if r == '&' {
				dat.mode = 1
			} else {
				dat.parsed.WriteRune(r)
			}
		} else if dat.mode == 1 {
			// mode: First char after &
			more := false
			holdRune = '�'
			holdStr.Reset()
			switch r {
			case '&':
				holdRune = '&'
			case '"':
				holdRune = '"'
			case '<':
				holdRune = '<'
			case '>':
				holdRune = '>'
			default:
				more = true
			}

			if more {
				dat.mode = 2
				holdStr.WriteRune(r)
				if !unicode.IsLetter(r) && !unicode.IsDigit(r) {
					panic("CharData parse: unexpected char (" + string(r) + ") in entity name")
				}
			} else {
				dat.parsed.WriteRune(holdRune)
				dat.mode = 0
				holdRune = '�'
			}
		} else if dat.mode == 2 {
			// mode: Entity name
			if r == ';' {
				dat.mode = 0
				entityName := holdStr.String()
				entity, has := defaultEntites[entityName]
				if !has {
					panic("CharData parse: unknown entity: " + entityName)
				}
				dat.parsed.WriteString(entity)
			} else if unicode.IsLetter(r) || unicode.IsDigit(r) {
				holdStr.WriteRune(r)
			} else {
				panic("CharData parse: forgot terminator in entity name: " + holdStr.String())
			}
		} else {
			panic("CharData parse: unknown mode: " + strconv.Itoa(dat.mode))
		}
	}
	if dat.mode != 0 {
		panic("CharData parse: finished in wrong mode: " + strconv.Itoa(dat.mode))
	}
	dat.isParsed = true
	return nil
}

func (dat *CharData) writeRune(r rune) error {
	_, err := dat.raw.WriteRune(r)
	return err
}

/// A Token is an interface holding one of the token types: StartElement, EndElement, CharData, Comment, EndInput.
type Token interface {
	String() string
}

// "id" => {$elemNS, "id"}, ":id" => {"tlnml", "id"}
type Name struct {
	Space, Local string
}

func (n Name) String() string {
	return n.Space + ":" + n.Local
}

type Attr struct {
	Name   Name
	Value  string
	Quoted bool
}

type AttrMap []Attr

func (m *AttrMap) Get(name NameOrString) (string, bool) {
	return "", false
}
func (m *AttrMap) Has(name NameOrString) bool {
	return false
}
func (m *AttrMap) Set(name NameOrString, value string) {

}

type Pos struct {
	Byte int
	Line int
	Col  int
}

func (this *Pos) advance(r rune, size int) {
	this.Byte += size
	this.Col += 1
	if r == '\n' {
		this.Line += 1
		this.Col = 0
	}
}

type ProcInst struct {
	Name     string
	Attr     AttrMap
	StartPos Pos
}

type BasicElement struct {
	Style    rune
	View     string
	Name     Name
	StartPos Pos
}

type StartElement struct {
	BasicElement
	SelfClosing bool
	Attr        AttrMap
}

type EndElement struct {
	BasicElement
	AbbrevEnding bool
}

type EndInput struct {
	StartPos Pos
}

type TokenReader interface {
	ReadToken() (Token, error)
}

func NewTokenReader(in io.Reader) TokenReader {
	return &BasicTokenReader{
		src:  bufio.NewReader(in),
		mode: 0,
	}
}

type BasicTokenReader struct {
	src    *bufio.Reader
	mode   int
	cur    Token
	pos    Pos
	stacks map[string][]Token
}

func (this *BasicTokenReader) ReadToken() (Token, error) {
	var err error

	if this.cur == nil {
		this.cur = new(CharData)
	}
	if this.stacks == nil {
		this.stacks = make(map[string][]Token)
		this.stacks[""] = make([]Token, 0)
	}
	for {
		// Read a single character from input
		c, c_size, err := this.src.ReadRune()
		this.pos.advance(c, c_size)
		if err == io.EOF {
			break
		}
		if err != nil {
			return this.cur, err
		}
		// Inline text mode
		if this.mode == 0 {
			this.cur.(*CharData).writeRune(c)
			continue
		}
	}

	err = nil
	switch v := this.cur.(type) {
	case *CharData:
		err = v.parse()
	}
	return this.cur, err
}
