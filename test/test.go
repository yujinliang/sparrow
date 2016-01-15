// test.go
package main

import (
	"fmt"

	"github.com/yujinliang/sparrow"
)

func main() {

	var sp sparrow.Sparrow
	sp.Initialize(nil)
	_, err := sp.Route2DB("db", "table", "user_id", "id0041", false, false)
	if err != nil {
		fmt.Println(err)
	}
	_, err = sp.Route2DB("db", "table", "user_id", "id0041", false, true)
	if err != nil {
		fmt.Println(err)
	}
	_, err = sp.Route2DB("db", "table", "user_id", "id0041", true, false)
	if err != nil {
		fmt.Println(err)
	}

}
