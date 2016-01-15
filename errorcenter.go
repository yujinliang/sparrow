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
	"errors"
)

var ErrDBNameEmpty = errors.New("DB Name Empty")
var ErrTableNameEmpty = errors.New("Table Name Empty")
var ErrDbShardKeyEmpty = errors.New("Shard Key Empty")
var ErrDbShardValueEmpty = errors.New("Shard Value Empty")
var ErrDbShardSchemeNotExist = errors.New("DB Shard Scheme Not Exist")
var ErrDbShardKeyNotMatch = errors.New("DB Shard Key NOT Match")
var ErrDbShardSchemeDbGroupEmpty = errors.New("DB Shard Scheme`s DB Group is Empty")
var ErrDbGroupNotExist = errors.New("DB Group Not Exists")
var ErrMasterDBKO = errors.New("Master DB KO")
var ErrSlaveDbOfDbGroupNotExits = errors.New("Slave DB Not Exists")
var ErrAllSlaveDbOfDbGroupKO = errors.New("Slave DB All KO")
