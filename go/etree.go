package tlnml

// /// A Node is an interface holding one of the types: CharData, Comment, ElemNode.
// type Node interface{}

// /// A NameOrString is an interface holding one of the types: Name, string.
// type NameOrString interface{}

// type ElemOrToken interface{}

// type MilestoneNode struct {
// 	Elem      *ElemNode
// }

// type ElemNode struct {
// 	NodeKind    rune
// 	Parent      *ElemNode
// 	View        string
// 	Name        Name
// 	Attrs       AttrMap
// 	Children    []*ElemOrToken
// }

// func (node ElemNode) Siblings() []*ElemNode {}

// // Negative Python-style indices are accepted.
// func (node ElemNode) GetChildAt(index int) *Node {}

// func (node *ElemNode) InsertChildAt(index int, child *Node) {}

// func (node *ElemNode) RemoveChildAt(index int) {}

// func (node ElemNode) GetElemById(id string) *ElemNode {}

// func (node ElemNode) GetElemsByTagName(name NameOrString) []*ElemNode {}

// /// The concatenation of the direct text children of this element.
// func (node ElemNode) InnerText() string {}

// /// The concatenation of the direct and indirect text children of this element.
// func (node ElemNode) Text() string {}

// type Document struct {
// 	Namespaces map[string]string
// 	Root *ElemNode
// }
