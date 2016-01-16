# Sparrow(小麻雀)
------
### 目的
> 主要用于学习，兴趣，代码逻辑直观，不跳步，当然如果用于生产环境,则代码需要优化强化，以及严格测试.
   
------

###  简述
> 一个简单的客户端哈希分库分表实现,追求简单，直观 ，用于mysql, mariadb等关系数据做水平扩展
以哈希方式，也不解析什么sql,那些高大上的东西太麻烦,一时也搞不定，在客户端实现分库分表，因为sql一般是固定的，
程序员自己分析sql,然后决定调用相应分库分表API,在它之上再写个数据层处理更高的逻辑以及缓存，以webservice方式基于http或tcp,
对外提供数据,这是我对它的定位，见笑,哈哈.

------    
### 要点
> 所有的分库分表以及节点信息都存放在etcd中， 客户端Watch远端etcd中配置的变化，及时更新本地缓存，etcd充当配置中心角色.

------
### 特性
> * 哈希方式分库分表
> * 读写分离
> * 数据库节点故障上报,存入etcd,供其它运维系统检查节点健康
> * 分库分表，及数据库节点配置信息皆存入etcd
> * 启动时主动拉取配置信息，并watch etcd以便配置变更时拉取更新

------
### 共同学习

> * @ sparrow - 2016
> * 作者: 悟空
> * 邮箱: htyu_0203_39@sina.com ; 285779289@qq.com
> * 心声：三人行，必有我师.
