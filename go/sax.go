package tlnml

import (
	"bufio"
	"errors"
	"io"
	"regexp"
	"strconv"
	"strings"
	"unicode"
	"fmt"
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
	val         strings.Builder
	isParsed    bool
	mode        int
	tmpRune rune
	tmpStr strings.Builder
}

func (dat CharData) String() string {
	return dat.val.String()
}

func (dat *CharData) writeRune(r rune) error {
	if dat.mode == 0 {
		// mode: Regular text
		if r == '&' {
			dat.mode = 1
		} else {
			dat.val.WriteRune(r)
		}
	} else if dat.mode == 1 {
		// mode: First char after &
		more := false
		dat.tmpRune = '�'
		dat.tmpStr.Reset()
		switch r {
		case '&':
			dat.tmpRune = '&'
		case '"':
			dat.tmpRune = '"'
		case '<':
			dat.tmpRune = '<'
		case '>':
			dat.tmpRune = '>'
		default:
			more = true
		}

		if more {
			dat.mode = 2
			dat.tmpStr.WriteRune(r)
			if !unicode.IsLetter(r) && !unicode.IsDigit(r) {
				return errors.New("CharData parse: unexpected char (" + string(r) + ") in entity name")
			}
		} else {
			dat.val.WriteRune(dat.tmpRune)
			dat.mode = 0
			dat.tmpRune = '�'
		}
	} else if dat.mode == 2 {
		// mode: Entity name
		if r == ';' {
			dat.mode = 0
			entityName := dat.tmpStr.String()
			entity, has := defaultEntites[entityName]
			if !has {
				return errors.New("CharData parse: unknown entity: " + entityName)
			}
			dat.val.WriteString(entity)
		} else if unicode.IsLetter(r) || unicode.IsDigit(r) {
			dat.tmpStr.WriteRune(r)
		} else {
			return errors.New("CharData parse: forgot terminator in entity name: " + dat.tmpStr.String())
		}
	} else {
		return errors.New("CharData parse: unknown mode: " + strconv.Itoa(dat.mode))
	}

	return nil
}

func (dat CharData) onEscape() bool {
	return dat.mode != 0
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

func (this ProcInst) String() string {
	return ""
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

func (this StartElement) String() string {
	return fmt.Sprintf("StartElement(%c %s%%%s %v)", this.Style, this.View, this.Name.String(), this.SelfClosing)
}

type EndElement struct {
	BasicElement
	AbbrevEnding bool
}

func (this EndElement) String() string {
	return fmt.Sprintf("EndElement(%c %s%%%s %v)", this.Style, this.View, this.Name.String(), this.AbbrevEnding)
}

type EndInput struct {
	StartPos Pos
}

func (this EndInput) String() string {
	return "␄"
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
	submode   int
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
	
	tmpStr := strings.Builder{}
	for {
		// Read a single character from input
		r, r_size, err := this.src.ReadRune()
		this.pos.advance(r, r_size)
		if err == io.EOF {
			break
		}
		if err != nil {
			return this.cur, err
		}
		// Inline text mode
		if this.mode == 0 {
			// mode: regular text
			cur := this.cur.(*CharData)
			if r == '<' && !cur.onEscape() {
				tmpStr.Reset()
				this.mode = 1
				// yield the current token
				break
			} else {
				cur.writeRune(r)
			}
		} else if this.mode == 1 {
			// mode: found <, not sure if start or end
			tmpStr.Reset()
			if r == '/' {
				this.mode = 3
				this.submode = 0
				cur := new(EndElement)
				cur.Style = XML_STYLE
				this.cur = cur
			} else {
				this.mode = 2
				this.submode = 0
				cur := new(StartElement)
				cur.Style = XML_STYLE
				tmpStr.WriteRune(r)
				this.cur = cur
			}
		} else if this.mode == 2 {
			// mode: start XML tag
			if this.submode == 0 {
				// submode: reading view and name
				if unicode.IsLetter(r) || unicode.IsDigit(r) || r == '|' || r == ':' {
					tmpStr.WriteRune(r)
				} else {
					// Separate name parts
					// Use regex: (([\p{L}\p{N}._-]+)\|)?(([\p{L}\p{N}._-]+?):|:|)([\p{L}\p{N}._-]+)
					// View: group 2
					// Namespace: group 3 (including colon at the end)
					// Namespace: group 4
					// Local: group 5
					println(tmpStr.String())
					this.submode = 1
				}
			} else if this.submode == 1 {
				// submode: reading attribute
			} else {
				return nil, errors.New("ReadToken: unknown submode: " + strconv.Itoa(this.submode))
			}
		} else if this.mode == 3 {
			// mode: end XML tag
		} else {
			return nil, errors.New("ReadToken: unknown mode: " + strconv.Itoa(this.mode))
		}

	}

	err = nil
	// switch v := this.cur.(type) {
	// case *CharData:
	// 	err = v.parse()
	// }
	return this.cur, err
}
