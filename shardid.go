package sparrow

type IShardIdGenerator interface {

	GenerateShardId(shardkey string) (shardid uint64, err error)
}

//default ShardIdGenerator implement.
type HashShardIdGenerator  struct{

	Shardkey string
	Shardid uint64

}

func(h *HashShardIdGenerator) GenerateShardId(shardkey string) (shardid uint64, err error) {

	return hashString2Number(shardkey), nil
}