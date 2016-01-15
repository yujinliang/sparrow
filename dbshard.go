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
	"fmt"
	"strconv"
)

//with it , caller must construct db connection , and sql execution and so on.
type DBShardInfo struct {
	DBNode    *DBAtom
	DBName    string
	TableName string
}

type Sparrow struct {
	CfgEndpoints []string //like: ip:port ; domainName:port
}

//get db shard config for remote config center.
func (s *Sparrow) Initialize(cfgEndpoints []string) error {

	//db group1
	DBAtomRepository = make(map[string]*DBAtom)
	DBAtomRepository["192.168.1.1:8080"] = &DBAtom{Ip: "192.168.1.1", Port: 8080, DBUser: "user", DBPwd: "123", DBFactor: 10, DBEnable: true, IsMaster: true}
	DBAtomRepository["192.168.1.2:8080"] = &DBAtom{Ip: "192.168.1.2", Port: 8080, DBUser: "user", DBPwd: "123", DBFactor: 8, DBEnable: true, IsMaster: false}
	DBAtomRepository["192.168.1.3:8080"] = &DBAtom{Ip: "192.168.1.3", Port: 8080, DBUser: "user", DBPwd: "123", DBFactor: 8, DBEnable: true, IsMaster: false}
	//db group2
	DBAtomRepository["192.168.1.4:8080"] = &DBAtom{Ip: "192.168.1.4", Port: 8080, DBUser: "user", DBPwd: "123", DBFactor: 10, DBEnable: true, IsMaster: true}
	DBAtomRepository["192.168.1.5:8080"] = &DBAtom{Ip: "192.168.1.5", Port: 8080, DBUser: "user", DBPwd: "123", DBFactor: 8, DBEnable: true, IsMaster: false}
	DBAtomRepository["192.168.1.6:8080"] = &DBAtom{Ip: "192.168.1.6", Port: 8080, DBUser: "user", DBPwd: "123", DBFactor: 8, DBEnable: true, IsMaster: false}
	//db group map
	DBGroupRepository = make(map[string]*DBGroup)
	DBGroupRepository["g1"] = &DBGroup{MasterDBAtomkey: "192.168.1.1:8080", SlaveDBAtomKeys: []string{"192.168.1.2:8080", "192.168.1.3:8080"}, SlaveSum: 2}
	DBGroupRepository["g2"] = &DBGroup{MasterDBAtomkey: "192.168.1.4:8080", SlaveDBAtomKeys: []string{"192.168.1.5:8080", "192.168.1.6:8080"}, SlaveSum: 2}
	//db scale out scheme map
	DBScaleOutSchemeRepository = make(map[string]*DBScaleOutScheme)
	DBScaleOutSchemeRepository["db"+"table"] = &DBScaleOutScheme{TableShardkey: "user_id", DBGroupSum: 2, TablePerDB: 4, DBGroupKeys: map[string]string{"0": "g1", "1": "g2"}}
	return nil

}

//when match case :shardKey == ? or shardKey != ?, then run this method, coz just only choose one db node to execute sql.
//use this method on 99.99%.
func (s *Sparrow) Route2DB(dbName string, tableName string, shardKey string, shardValue string, forceMaster bool, isWrite bool) (*DBShardInfo, error) {

	if len(dbName) <= 0 {
		return nil, ErrDBNameEmpty
	}
	if len(tableName) <= 0 {
		return nil, ErrTableNameEmpty
	}
	if len(shardKey) <= 0 {
		return nil, ErrDbShardKeyEmpty
	}
	if len(shardValue) <= 0 {
		return nil, ErrDbShardValueEmpty
	}
	//TODO: check initialization is done?
	dbShardScheme := DBScaleOutSchemeRepository[dbName+tableName]
	if dbShardScheme == nil {
		return nil, ErrDbShardSchemeNotExist
	}
	if shardKey != dbShardScheme.TableShardkey {
		return nil, ErrDbShardKeyNotMatch
	}
	if dbShardScheme.DBGroupSum <= 0 {
		return nil, ErrDbShardSchemeDbGroupEmpty
	}
	//lookup the db group, just mod.
	shardNumber := hashString2Number(shardValue)
	groupIndex := shardNumber % uint64(dbShardScheme.DBGroupSum)
	groupKey := dbShardScheme.DBGroupKeys[strconv.FormatUint(uint64(groupIndex), 10)]
	//just for debug.
	fmt.Printf("shardNumber:%v, groupIndex:%v, groupKey: %v\n", shardNumber, groupIndex, groupKey)
	//--
	dbGroup := DBGroupRepository[groupKey]
	if dbGroup == nil {
		return nil, ErrDbGroupNotExist
	}

	var dbNode *DBAtom = nil
	if forceMaster || isWrite {

		dbNode = DBAtomRepository[dbGroup.MasterDBAtomkey]
		if !dbNode.DBEnable {

			return nil, ErrMasterDBKO
		}

	} else {

		if dbGroup.SlaveSum > 0 && len(dbGroup.SlaveDBAtomKeys) > 0 {

			dbNodeIndex := randIntRange(0, int(dbGroup.SlaveSum))
			nextDBNodeIndex := 0
			dbNodeKey := dbGroup.SlaveDBAtomKeys[dbNodeIndex]
			dbNode = DBAtomRepository[dbNodeKey]
			if !dbNode.DBEnable {

				distanceOfMax := dbGroup.SlaveSum - uint(dbNodeIndex)
				if (float32(distanceOfMax) / float32(dbGroup.SlaveSum)) > randRangePoint {

					nextDBNodeIndex = randIntRange(0, int(dbNodeIndex))

				} else {

					nextDBNodeIndex = randIntRange(int(dbNodeIndex), int(dbGroup.SlaveSum))
				}
				dbNodeKey = dbGroup.SlaveDBAtomKeys[nextDBNodeIndex]
				dbNode = DBAtomRepository[dbNodeKey]
				if !dbNode.DBEnable {
					//choose ajacent slave node.
					for i := 0; i < int(dbGroup.SlaveSum); i++ {
						if i == dbNodeIndex || i == nextDBNodeIndex {
							continue
						}
						dbNodeKey = dbGroup.SlaveDBAtomKeys[i]
						dbNode = DBAtomRepository[dbNodeKey]
						if dbNode.DBEnable {
							break
						}
					}

				}
			}

		} else {

			return nil, ErrSlaveDbOfDbGroupNotExits
		}
	}

	if !dbNode.DBEnable {

		return nil, ErrAllSlaveDbOfDbGroupKO
	}
	tableIndex := shardNumber % uint64(dbShardScheme.TablePerDB)
	realTableName := tableName + strconv.FormatUint(uint64(tableIndex), 10)
	dbInfo := &DBShardInfo{DBNode: dbNode, DBName: dbName, TableName: realTableName}

	return dbInfo, nil

}

//when match case: shardKey >, >=, <, <= in some range, then run this method, coz to get all db node to execute sql.
//do not support to use this method, unless you know what you are doing.
//regarding complex search , must use search engine , like: solr.
func (s *Sparrow) Route2DBs(dbName string, tableName string, shardKey string, forceMaster bool, isWrite bool) ([]*DBShardInfo, error) {

	return nil, nil

}

//when the db node meet some error, can not work, then use this method to report remote config center to check it health.
func (s *Sparrow) ComplainDBNode(nodeCfg *DBShardInfo) error {

	return nil
}
