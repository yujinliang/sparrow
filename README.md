# sparrow(小麻雀)
------
> 一个简单的客户端哈希分库分表实现,追求简单，直观 ，用于mysql, mariadb等关系数据做水平扩展
以哈希方式，也不解析什么sql,那些高大上的东西太麻烦,一时也搞不定，在客户端实现分库分表，因为sql一般是固定的，
程序员自己分析sql,然后决定调用相应分库分表API,在它之上再写个数据层处理更高的逻辑以及缓存，以webservice方式基于http或tcp,
对外提供数据,这是我对它的定位，见笑,哈哈.

------    
### 要点
> 所有的分库分表以及节点信息都存放在etcd中， 客户端Watch远端etcd中配置的变化，及时更新本地缓存，etcd充当配置中心角色.
