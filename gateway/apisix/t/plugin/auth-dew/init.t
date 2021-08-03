use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;
log_level('debug');

__DATA__

=== TEST 1: test resource
--- config
    location /t {
        content_by_lua_block {
            local json = require("cjson")
            local m_init = require("apisix.plugins.auth-dew.init")
            local m_resource = require("apisix.plugins.auth-dew.resource")
            local m_redis = require("apisix.plugins.auth-dew.redis")
            local m_utils = require("apisix.plugins.auth-dew.utils")

            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.hset("dew:iam:resources","api://app1.tenant1/p1?a=1##get","{\"account_codes\":\"#acc1#\"}")
            m_redis.hset("dew:iam:resources","api://app1.tenant1/p1?a=2##get","{\"account_codes\":\"#acc2#\"}")
            m_redis.hset("dew:iam:resources","api://app1.tenant1/p1?a=3##get","{\"account_codes\":\"#acc3#\"}")
            m_redis.hset("dew:iam:resources","api://app1.tenant1/p1?a=4##get","{\"account_codes\":\"#acc4#\"}")
            m_redis.hset("dew:iam:resources","api://app1.tenant1/p1?a=5##get","{\"account_codes\":\"#acc5#\"}")

            m_init.init("dew:iam:resources","dew:iam:change_resources",5)

            local resources = m_resource.get_res()
            ngx.say(resources["api"]["app1"]["tenant1"]["p1"]["?"]["a=1"]["$"]["get"]["uri"])
            ngx.say(resources["api"]["app1"]["tenant1"]["p1"]["?"]["a=5"]["$"]["get"]["uri"])

            m_redis.hset("dew:iam:change_resources","api://app1.tenant1/p1?a=6##get","{\"account_codes\":\"#acc6#\"}")
            m_redis.hset("dew:iam:change_resources","api://app1.tenant1/p1?a=7##get","{\"account_codes\":\"#acc7#\"}")
            m_redis.hset("dew:iam:change_resources","api://app1.tenant1/p1?a=1##get","")

            m_redis.hscan("dew:iam:change_resources", "*", 10, function(k, v)
               local res = m_utils.split(k, "##")
               if (v ~= "") then
                   m_resource.add_res(res[2], res[1], json.decode(v))
               else
                   m_resource.remove_res(res[2], res[1])
               end
            end)

            local resources = m_resource.get_res()
            ngx.say(resources["api"]["app1"]["tenant1"]["p1"]["?"]["a=1"])
            ngx.say(resources["api"]["app1"]["tenant1"]["p1"]["?"]["a=6"]["$"]["get"]["uri"])
            ngx.say(resources["api"]["app1"]["tenant1"]["p1"]["?"]["a=7"]["$"]["get"]["uri"])
        }
    }
--- request
GET /t
--- response_body
api://app1.tenant1/p1?a=1
api://app1.tenant1/p1?a=5
nil
api://app1.tenant1/p1?a=6
api://app1.tenant1/p1?a=7
--- no_error_log
[error]

