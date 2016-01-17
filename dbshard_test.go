/*
@author:yujinliang
@email:285779289@qq.com
@date:2015-1-11
@desc:simple db shard client side sdk.
@just for fun , study.
Copyright 2015 The Sparrow Authors All rights reserved.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
package sparrow

import (
	"testing"
)

func TestRoute2DB1(t *testing.T) {

	var sp Sparrow
	sp.Initialize(nil)
	rc, err := sp.Route2DB("db", "table", "user_id", "id0041", false, false)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
	}
	if rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should not be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	//--
	rc, err = sp.Route2DB("db", "table", "user_id", "id0041", false, true)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	if !rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	//--
	rc, err = sp.Route2DB("db", "table", "user_id", "id0041", true, false)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	if !rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	//--
	rc, err = sp.Route2DB("db", "table", "user_id", "id0041", true, true)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	if !rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	//--
	rc, err = sp.Route2DB("db", "table", "user_id", "id0941", true, true)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	if !rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	//--
	rc, err = sp.Route2DB("db", "table", "user_id", "id0941", false, false)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	if rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should not be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	//--
	rc, err = sp.Route2DB("db", "table", "user_id", "id0941", false, true)
	if err != nil {

		t.Error(err)

	}
	if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

		t.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)

	}
	if !rc.DBNode.IsMaster {

		t.Errorf("dbname: %s, tablename: %d, should be master.", rc.DBNode.Ip, rc.DBNode.Port)
	}
	if !rc.DBNode.DBEnable {

		t.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
	}
}

func BenchmarkRoute2DB1(b *testing.B) {

	b.StopTimer()
	var sp Sparrow
	sp.Initialize(nil)
	b.StartTimer()
	for i := 0; i < b.N; i++ {

		rc, err := sp.Route2DB("db", "table", "user_id", "id0041", false, false)
		if err != nil {

			b.Error(err)

		}
		if rc == nil || rc.DBNode == nil || len(rc.DBName) <= 0 || len(rc.TableName) <= 0 {

			b.Errorf("dbatom:%v, dbname: %v, tablename:%v\n", rc.DBNode, rc.DBName, rc.TableName)
		}
		if rc.DBNode.IsMaster {

			b.Errorf("dbname: %s, tablename: %d, should not be master.", rc.DBNode.Ip, rc.DBNode.Port)
		}
		if !rc.DBNode.DBEnable {

			b.Errorf("dbname: %s, tablename: %d, disable.", rc.DBNode.Ip, rc.DBNode.Port)
		}
	}
}

func TestRoute2DBs1(t *testing.T) {

	var sp Sparrow
	sp.Initialize(nil)
	rc, err := sp.Route2DBs("db", "table", "user_id", false, false)
	if err != nil {

		t.Error(err)
	}

	if rc == nil || len(rc) <= 0 {

		t.Errorf("%s, %s, result empty", "db", "table")
	}

	for i, v := range rc {

		if v == nil || v.DBNode == nil {

			t.Errorf("%s, %s, idx: %d empty", "db", "talbe", i)
		}
		if v.DBNode.IsMaster {

			t.Errorf("%s, %d should not be master", v.DBNode.Ip, v.DBNode.Port)
		}
		if !v.DBNode.DBEnable {

			t.Errorf("%s, %d disable", v.DBNode.Ip, v.DBNode.Port)
		}
		t.Logf("nf, ro:=> %s, %s, %s, %d, %s, %s, Factor: %d, Enable: %b, isMaster: %b", v.DBName, v.TableName, v.DBNode.Ip, v.DBNode.Port, v.DBNode.DBUser, v.DBNode.DBPwd, v.DBNode.DBFactor, v.DBNode.DBEnable, v.DBNode.IsMaster)
	}
	//--
	rc, err = sp.Route2DBs("db", "table", "user_id", false, true)
	if err != nil {

		t.Error(err)
	}

	if rc == nil || len(rc) <= 0 {

		t.Errorf("%s, %s, result empty", "db", "table")
	}

	for i, v := range rc {

		if v == nil || v.DBNode == nil {

			t.Errorf("%s, %s, idx: %d empty", "db", "talbe", i)
		}
		if !v.DBNode.IsMaster {

			t.Errorf("%s, %d should be master", v.DBNode.Ip, v.DBNode.Port)
		}
		if !v.DBNode.DBEnable {

			t.Errorf("%s, %d disable", v.DBNode.Ip, v.DBNode.Port)
		}
		t.Logf("nf, w:=> %s, %s, %s, %d, %s, %s, Factor: %d, Enable: %b, isMaster: %b", v.DBName, v.TableName, v.DBNode.Ip, v.DBNode.Port, v.DBNode.DBUser, v.DBNode.DBPwd, v.DBNode.DBFactor, v.DBNode.DBEnable, v.DBNode.IsMaster)
	}
	//--
	rc, err = sp.Route2DBs("db", "table", "user_id", true, false)
	if err != nil {

		t.Error(err)
	}

	if rc == nil || len(rc) <= 0 {

		t.Errorf("%s, %s, result empty", "db", "table")
	}

	for i, v := range rc {

		if v == nil || v.DBNode == nil {

			t.Errorf("%s, %s, idx: %d empty", "db", "talbe", i)
		}
		if !v.DBNode.IsMaster {

			t.Errorf("%s, %d should be master", v.DBNode.Ip, v.DBNode.Port)
		}
		if !v.DBNode.DBEnable {

			t.Errorf("%s, %d disable", v.DBNode.Ip, v.DBNode.Port)
		}
		t.Logf("f, ro:=> %s, %s, %s, %d, %s, %s, Factor: %d, Enable: %b, isMaster: %b", v.DBName, v.TableName, v.DBNode.Ip, v.DBNode.Port, v.DBNode.DBUser, v.DBNode.DBPwd, v.DBNode.DBFactor, v.DBNode.DBEnable, v.DBNode.IsMaster)
	}
	//--
	rc, err = sp.Route2DBs("db", "table", "user_id", true, true)
	if err != nil {

		t.Error(err)
	}

	if rc == nil || len(rc) <= 0 {

		t.Errorf("%s, %s, result empty", "db", "table")
	}

	for i, v := range rc {

		if v == nil || v.DBNode == nil {

			t.Errorf("%s, %s, idx: %d empty", "db", "talbe", i)
		}
		if !v.DBNode.IsMaster {

			t.Errorf("%s, %d should  be master", v.DBNode.Ip, v.DBNode.Port)
		}
		if !v.DBNode.DBEnable {

			t.Errorf("%s, %d disable", v.DBNode.Ip, v.DBNode.Port)
		}
		t.Logf("f, w:=> %s, %s, %s, %d, %s, %s, Factor: %d, Enable: %b, isMaster: %b", v.DBName, v.TableName, v.DBNode.Ip, v.DBNode.Port, v.DBNode.DBUser, v.DBNode.DBPwd, v.DBNode.DBFactor, v.DBNode.DBEnable, v.DBNode.IsMaster)
	}
}

func BenchmarkRoute2DBs1(b *testing.B) {

}
