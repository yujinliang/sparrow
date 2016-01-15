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
	"hash/fnv"
	"math/rand"
	"time"
)

func hashString2Number(s string) uint64 {

	h := fnv.New64a()
	h.Write([]byte(s))
	return h.Sum64()
}

func randIntRange(min int, max int) int {

	if max <= min {
		return min
	}
	rand.Seed(time.Now().UTC().UnixNano())
	return min + rand.Intn(max-min)
}

func lookUpSlaveOfDBGroup(dbGroup *DBGroup) (*DBAtom, error) {

	var dbNode *DBAtom = nil
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

	if !dbNode.DBEnable {

		return nil, ErrAllSlaveDbOfDbGroupKO
	}

	return dbNode, nil

}
