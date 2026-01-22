"""
gRPC客户端示例 - Python

安装依赖:
pip install grpcio grpcio-tools

生成Python客户端代码:
python -m grpc_tools.protoc -I./proto --python_out=. --grpc_python_out=. proto/sql_parser.proto
"""

import grpc
import sql_parser_pb2
import sql_parser_pb2_grpc
import json


def main():
    # 连接到gRPC服务器
    channel = grpc.insecure_channel('localhost:50051')
    stub = sql_parser_pb2_grpc.SqlParserServiceStub(channel)

    print("=" * 50)
    print("gRPC客户端测试 - Python")
    print("=" * 50)
    print()

    # 测试1: Health Check
    print("测试1: Health Check")
    try:
        response = stub.HealthCheck(sql_parser_pb2.HealthCheckRequest())
        print(f"  状态: {response.status}")
        print(f"  版本: {response.version}")
        print()
    except grpc.RpcError as e:
        print(f"  错误: {e.details()}")
        print()

    # 测试2: Parse SQL
    print("测试2: Parse SQL")
    try:
        request = sql_parser_pb2.ParseSqlRequest(
            sql="SELECT * FROM users WHERE id = 123 AND name = 'John'",
            dialect="mysql",
            no_cache=False
        )
        response = stub.ParseSql(request)
        
        if response.HasField('success'):
            success = response.success
            print(f"  ✅ 解析成功")
            print(f"  缓存: {'是' if success.cached else '否'}")
            print(f"  耗时: {success.elapsed_ms:.2f}ms")
            print(f"  AST (前200字符): {success.ast_json[:200]}...")
        else:
            error = response.error
            print(f"  ❌ 解析失败: {error.error_message}")
            print(f"  耗时: {error.elapsed_ms:.2f}ms")
        print()
    except grpc.RpcError as e:
        print(f"  错误: {e.details()}")
        print()

    # 测试3: Generate Fingerprint
    print("测试3: Generate Fingerprint")
    try:
        request = sql_parser_pb2.FingerprintRequest(
            sql="SELECT * FROM users WHERE id = 123 AND age IN (25, 30, 35, 40, 45)",
            dialect="mysql",
            max_in_values=3
        )
        response = stub.GenerateFingerprint(request)
        
        if response.HasField('success'):
            success = response.success
            print(f"  ✅ 指纹生成成功")
            print(f"  指纹: {success.fingerprint}")
            print(f"  耗时: {success.elapsed_ms:.2f}ms")
        else:
            error = response.error
            print(f"  ❌ 指纹生成失败: {error.error_message}")
            print(f"  耗时: {error.elapsed_ms:.2f}ms")
        print()
    except grpc.RpcError as e:
        print(f"  错误: {e.details()}")
        print()

    # 测试4: UPDATE语句指纹
    print("测试4: UPDATE语句指纹")
    try:
        request = sql_parser_pb2.FingerprintRequest(
            sql="UPDATE products SET price = 99.99, quantity = 100 WHERE category = 'electronics'",
            dialect="mysql",
            max_in_values=0
        )
        response = stub.GenerateFingerprint(request)
        
        if response.HasField('success'):
            success = response.success
            print(f"  ✅ 指纹生成成功")
            print(f"  指纹: {success.fingerprint}")
            print(f"  耗时: {success.elapsed_ms:.2f}ms")
        else:
            error = response.error
            print(f"  ❌ 指纹生成失败: {error.error_message}")
        print()
    except grpc.RpcError as e:
        print(f"  错误: {e.details()}")
        print()

    print("=" * 50)
    print("所有测试完成！")
    print("=" * 50)

    channel.close()


if __name__ == '__main__':
    main()
