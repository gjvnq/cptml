package tlnml

import (
	"bufio"
	"errors"
	"fmt"
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

var reTagName = regexp.MustCompile(`((?P<View>[\p{L}\p{N}._-]+)\|)?(?P<NSColon>(?P<NS>[\p{L}\p{N}._-]+?):|:|)(?P<Name>[\p{L}\p{N}._-]+)?`)

var NonParsedCharData = errors.New("CharData was not parsed")

type NameOrString interface{}

type Comment string
type CharData struct {
	/// Number of brackets used/necessary to escape the text. Zero if "normal" inline text.
	bracketsNum int
	val         strings.Builder
	mode        int
	tmpRune     rune
	tmpStr      strings.Builder
	final       string
	gotFirst    bool
	last        int
	StartPos    Pos
}

func (dat CharData) String() string {
	return dat.final
}

func (dat CharData) isZero() bool {
	return dat.val.Len() == 0
}

func (dat *CharData) finish() {
	dat.final = dat.val.String()[:dat.last]
	dat.val.Reset()
}

func (dat *CharData) writeRune(r rune) error {
	if dat.mode == 0 {
		// mode: Regular text
		isSpace := unicode.IsSpace(r)

		if r == '&' {
			dat.gotFirst = true
			dat.mode = 1
		} else if dat.gotFirst {
			dat.val.WriteRune(r)
		} else if !isSpace {
			dat.gotFirst = true
			dat.val.WriteRune(r)
		}
		if !isSpace && r != '&' {
			dat.last = dat.val.Len()
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
			dat.last = dat.val.Len()
		} else {
			dat.val.WriteRune(dat.tmpRune)
			dat.last = dat.val.Len()
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
			dat.last = dat.val.Len()
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
	isZero() bool
	finish()
}

// "id" => {$elemNS, "id"}, ":id" => {"tlnml", "id"}
type Name struct {
	Space, Local string
}

func (n Name) IsZero() bool {
	return len(n.Space) == 0 && len(n.Local) == 0
}

func (n Name) String() string {
	// if len(n.Space) == 0 {
	// 	return n.Local
	// }
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

func (this Pos) String() string {
	return fmt.Sprintf("%d:%d", this.Line, this.Col)
}

func (this Pos) IsZero() bool {
	return this.Byte == 0 && this.Line == 0 && this.Col == 0
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

func (this BasicElement) isZero() bool {
	return false
}

func (this BasicElement) finish() {}

func (this *BasicElement) setNameAndView(tag string) {
	nameParts := reTagName.FindStringSubmatch(tag)
	this.View = nameParts[2]
	this.Name.Space = nameParts[4]
	if len(nameParts[3]) == 1 {
		this.Name.Space = "tlnml"
	}
	this.Name.Local = nameParts[5]
}

type StartElement struct {
	BasicElement
	SelfClosing bool
	Attr        AttrMap
}

func (this StartElement) String() string {
	v := ""
	if len(this.View) != 0 {
		v = this.View + "|"
	}
	pos := ""
	if !this.StartPos.IsZero() {
		pos = " " + this.StartPos.String()
	}
	return fmt.Sprintf("StartElement('%c' %s%s %v%s)", this.Style, v, this.Name.String(), this.SelfClosing, pos)
}

type EndElement struct {
	BasicElement
	AbbrevEnding bool
}

func (this EndElement) String() string {
	v := ""
	if len(this.View) != 0 {
		v = this.View + "|"
	}
	pos := ""
	if !this.StartPos.IsZero() {
		pos = " " + this.StartPos.String()
	}
	return fmt.Sprintf("EndElement('%c' %s%s %v%s)", this.Style, v, this.Name.String(), this.AbbrevEnding, pos)
}

type EndInput struct {
	StartPos Pos
}

func (this EndInput) finish() {}

func (this EndInput) isZero() bool {
	return false
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
	src        *bufio.Reader
	mode       int
	submode    int
	cur        Token
	pos        Pos
	eof        bool
	stacks     map[string][]StartElement
	cleanToken bool
}

func (this *BasicTokenReader) push(elem StartElement) {
	if this.stacks[elem.View] == nil {
		this.stacks[elem.View] = make([]StartElement, 0)
	}
	this.stacks[elem.View] = append(this.stacks[elem.View], elem)
}

func (this *BasicTokenReader) pop(view string) StartElement {
	top := len(this.stacks[view])
	ans := this.stacks[view][top-1]
	this.stacks[view] = this.stacks[view][:top-1]
	return ans
}

func (this *BasicTokenReader) ReadToken() (Token, error) {
	var err error
	if this.stacks == nil {
		this.stacks = make(map[string][]StartElement)
		this.stacks[""] = make([]StartElement, 0)
	}

	tmpStr := strings.Builder{}
	r := '\000'
	r_size := 0
	skipRead := false
	for !this.eof {
		// Read a single character from input
		if !skipRead {
			r, r_size, err = this.src.ReadRune()
			this.pos.advance(r, r_size)
			// fmt.Printf("%c %d:%d %+v\n", r, this.mode, this.submode, this.cur)
			if err == io.EOF {
				this.eof = true
				// Yield current token if non-zero
				if this.cur != nil && this.cleanToken && !this.cur.isZero() {
					this.cur.finish()
					return this.cur, nil
				} else {
					return EndInput{}, nil
				}
			}
			if err != nil {
				return this.cur, err
			}
		}
		skipRead = false
		// Inline text mode
		if this.mode == 0 {
			// mode: regular text
			if !this.cleanToken {
				this.cleanToken = true
				curNew := new(CharData)
				curNew.StartPos = this.pos
				this.cur = curNew
			}
			cur := this.cur.(*CharData)
			if r == '<' && !cur.onEscape() {
				tmpStr.Reset()
				this.mode = 1
				this.cleanToken = false
				if !this.cur.isZero() {
					// yield the current token
					this.cur.finish()
					return this.cur, nil
				}
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
				cur.StartPos = this.pos
				cur.StartPos.Col -= 1
				this.cur = cur
				this.cleanToken = true
			} else {
				this.mode = 2
				this.submode = 0
				cur := new(StartElement)
				cur.Style = XML_STYLE
				cur.StartPos = this.pos
				cur.StartPos.Col -= 1
				tmpStr.WriteRune(r)
				this.cur = cur
				this.cleanToken = true
			}
		} else if this.mode == 2 {
			// mode: start XML tag
			if !this.cleanToken {
				this.cleanToken = true
				this.cur = new(StartElement)
			}
			cur := this.cur.(*StartElement)
			if this.submode == 0 {
				// submode: reading view and name
				if unicode.IsLetter(r) || unicode.IsDigit(r) || r == '|' || r == ':' {
					// Continue reading tag name
					tmpStr.WriteRune(r)
				} else {
					// Parse tag name
					cur.setNameAndView(tmpStr.String())
					this.submode = 1
					skipRead = true
				}
			} else if this.submode == 1 {
				// submode: reading attribute
				if r == '/' {
					cur.SelfClosing = true
					this.submode = 2
				}
				if r == '>' {
					if !cur.SelfClosing {
						this.push(*cur)
					}
					// yield
					this.mode = 0
					this.cleanToken = false
					this.cur.finish()
					return this.cur, nil
				}
			} else if this.submode == 2 {
				// submode: finish tag
				if r == '>' {
					if !cur.SelfClosing {
						this.push(*cur)
					}
					// yield
					this.mode = 0
					this.cleanToken = false
					this.cur.finish()
					return this.cur, nil
				} else {
					return nil, errors.New("ReadToken: unexpected char on finish XML tag: " + string(r))
				}
			} else {
				return nil, errors.New("ReadToken: unknown submode: " + strconv.Itoa(this.submode))
			}
		} else if this.mode == 3 {
			// mode: end XML tag
			cur := this.cur.(*EndElement)
			// submode: reading view and name
			if unicode.IsLetter(r) || unicode.IsDigit(r) || r == '|' || r == ':' {
				// Continue reading tag name
				tmpStr.WriteRune(r)
			} else if tmpStr.Len() != 0 {
				// Parse tag name
				cur.setNameAndView(tmpStr.String())
			}
			if r == '>' {
				var err error
				cur.AbbrevEnding = cur.Name.IsZero()
				start := this.pop(cur.View)
				if cur.AbbrevEnding {
					cur.Name = start.Name
					cur.View = start.View
				} else if cur.Name.String() != start.Name.String() {
					err = errors.New("wrong element closure, expected "+start.Name.String()+" but got "+cur.Name.String())
				}

				// yield
				this.mode = 0
				this.cleanToken = false
				this.cur.finish()
				return this.cur, err
			}
		} else {
			return nil, errors.New("ReadToken: unknown mode: " + strconv.Itoa(this.mode))
		}

	}

	return EndInput{}, nil
}
