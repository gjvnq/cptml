package tlnml

import (
	"github.com/stretchr/testify/assert"
	"strings"
	"testing"
	// "fmt"
)

func TestBasicTokenReader1(t *testing.T) {
	reader := NewTokenReader(strings.NewReader(""))
	token, err := reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader2(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("\tHi! "))
	token, err := reader.ReadToken()
	assert.Equal(t, "Hi!", token.String())
	assert.Nil(t, err)
	token, err = reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader3(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("Hi! &< &> && &t; &v; \v"))
	token, err := reader.ReadToken()
	assert.Equal(t, "Hi! < > & \t \v", token.String())
	assert.Nil(t, err)
	token, err = reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader4(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("Hi! &< &> && &v; &Tab; &sp;&pm;&LeftVectorBar;&Tab; "))
	token, err := reader.ReadToken()
	assert.Equal(t, "Hi! < > & \v \t  ±⥒\t", token.String())
	assert.Nil(t, err)
	token, err = reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader5(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("Hi! &< <:a/>\n abc <v|1ns:loc4l/>"))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "Hi! <", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' tlnml:a true 0:9)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "abc", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' v%1ns:loc4l true 1:7)", token.String())
	token, err = reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}
