/*
    为每一个mysql command and sql statement 相应执行入口。
    dispatcher 调用analyer{sql}  / router, 生成Plan, 然后统一交给executor执行。 
*/