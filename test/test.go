// test.go
package main

import (
	"fmt"

	"github.com/yujinliang/sparrow"
)

func main() {

	var sp sparrow.Sparrow
	sp.Initialize(nil)
	rc, err := sp.Route2DB("db", "table", "user_id", "id0041", false, false)
	if err != nil {
		fmt.Println(err)
	} else {

		fmt.Printf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
	}
	rc, err = sp.Route2DB("db", "table", "user_id", "id0041", false, true)
	if err != nil {
		fmt.Println(err)
	} else {

		fmt.Printf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	rc, err = sp.Route2DB("db", "table", "user_id", "id0041", true, false)
	if err != nil {
		fmt.Println(err)
	} else {

		fmt.Printf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
	}
	rc, err = sp.Route2DB("db", "table", "user_id", "id0041", true, true)
	if err != nil {
		fmt.Println(err)
	} else {

		fmt.Printf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
	}
	rc, err = sp.Route2DB("db1", "table", "user_id", "id0041", true, true)
	if err != nil {
		fmt.Println(err)
	}
	rc, err = sp.Route2DB("db", "table1", "user_id", "id0041", true, true)
	if err != nil {
		fmt.Println(err)
	}
	rc, err = sp.Route2DB("db", "table", "doc_id", "id0041", true, true)
	if err != nil {
		fmt.Println(err)
	}
	rc, err = sp.Route2DB("db", "table", "user_id", "id0091", true, true)
	if err != nil {
		fmt.Println(err)
	} else {

		fmt.Printf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
	}
	rc, err = sp.Route2DB("db", "table", "user_id", "id0091", false, false)
	if err != nil {
		fmt.Println(err)
	} else {

		fmt.Printf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
	}
}
