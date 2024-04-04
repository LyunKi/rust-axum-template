rust 服务端模板
技术栈：

1. server framework: axum.rs
2. orm: sea-orm postgres
3. 缓存: redis

## 启动 migration

配置 $DATABASE_URL 环境变量, 执行 `sea-orm-cli migrate up`

## 生成数据库实体

执行`sea-orm-cli generate entity -o entity/src -l`
