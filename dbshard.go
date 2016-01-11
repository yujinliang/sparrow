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

	return nil

}

//when match case :shardKey == ? or shardKey != ?, then run this method, coz just only choose one db node to execute sql.
//use this method on 99.99%.
func (s *Sparrow) Route2DB(dbName string, tableName string, shardKey string, forceMaster bool, isWrite bool) (*DBShardInfo, error) {

	return nil, nil
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
