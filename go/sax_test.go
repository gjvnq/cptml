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
	reader := NewTokenReader(strings.NewReader("Hi! &< <:a/>\n a<b/>c <v|1ns:loc4l/>"))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "Hi! <", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' tlnml:a true 0:8)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "a", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' b true 1:3)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "c", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' v|1ns:loc4l true 1:9)", token.String())
	token, err = reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}

func TestBasicTokenReader6(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("<a><b/></a>"))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' a false 0:1)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' b true 0:4)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' a false 0:8)", token.String())
}

func TestBasicTokenReader7(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("<a><b/></>"))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' a false 0:1)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' b true 0:4)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' a true 0:8)", token.String())
}

func TestBasicTokenReader8(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("<a><b><c><d></></></b></>"))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' a false 0:1)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' b false 0:4)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' c false 0:7)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' d false 0:10)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' d true 0:13)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' c true 0:16)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' b false 0:19)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' a true 0:23)", token.String())
}

func TestBasicTokenReader9(t *testing.T) {
	reader := NewTokenReader(strings.NewReader("<a><b><v|z><c><d></></></b></v|></>"))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' a false 0:1)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' b false 0:4)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' v|z false 0:7)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' c false 0:12)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' d false 0:15)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' d true 0:18)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' c true 0:21)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' b false 0:24)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' v|z true 0:28)", token.String())
	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' a true 0:33)", token.String())
}

func TestBasicTokenReader10(t *testing.T) {
	reader := NewTokenReader(strings.NewReader(`\h1{My&sp;<a|s>Title}\p{My Te</a|>xt!}`))
	token, err := reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('\\' h1 false 0:1)", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "My ", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('<' a|s false 0:11)", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "Title", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('\\' h1 true 0:21)", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "StartElement('\\' p false 0:22)", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "My Te", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('<' a|s true 0:30)", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "xt!", token.String())

	token, err = reader.ReadToken()
	assert.Nil(t, err)
	assert.Equal(t, "EndElement('\\' p true 0:38)", token.String())

	token, err = reader.ReadToken()
	assert.Equal(t, "␄", token.String())
	assert.Nil(t, err)
}
