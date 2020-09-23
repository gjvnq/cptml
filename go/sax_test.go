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
	assert.Equal(t, "", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader2(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("Hi!"))
	token, err := reader.ReadToken()
	assert.Equal(t, "Hi!", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader3(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("Hi! &< &> && &t; &v;"))
	token, err := reader.ReadToken()
	assert.Equal(t, "Hi! < > & \t \v", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader4(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("Hi! &< &> && &v; &Tab; &sp;&pm;&LeftVectorBar;"))
	token, err := reader.ReadToken()
	assert.Equal(t, "Hi! < > & \v \t  ±⥒", token.String())
	assert.Nil(t, err)
}
