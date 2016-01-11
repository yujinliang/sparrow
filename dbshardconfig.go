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

//below struct for config relation.

type DBAtom struct {
	Ip       string
	Port     uint
	DBUser   string
	DBPwd    string
	DBFactor int8 //db node performance priority, with number increase from 1 to 127, the performance premote
	DBEnable bool //if false , disable node to be access[write,read].
	IsMaster bool //default: false
}

var DBAtomRepository [string]*DBAtom

type DBGroup struct {
	MasterDBAtomkey string
	SlaveDBAtomKeys []string
	SlaveSum        uint
}

var DBGroupRepository [string]*DBGroup

type DBScaleOutScheme struct {
	TableShardkey string
	DBGroupSum    uint           //to be mod for DBGroup.
	TablePerDB    uint           //to be mod for real table name.
	DBGroupKeys   [string]string //key: unique hash number to identiy a db group, value: DBGroupKey
}

var DBScaleOutSchemeRepository [string]*DBScaleOutScheme //key: dbname + tablename
