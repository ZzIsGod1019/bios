/**

# EDA测试计划

## 测试目标

1. **消息队列测试**
   - 测试消息队列在不同数据体积下的消息发送和接收能力，确保系统能够正确处理小数据包和大数据包。
   - 验证消息队列的顺序性、一致性和持久性，确保消息不会丢失、重复或乱序。
   - 测试多节点环境下消息队列的顺序性、一致性和持久性，确保消息在多节点之间正常传递。
   - 测试消息持久化后，在不同负载条件下队列的吞吐量和响应时间。
   - 验证消息队列在高并发条件下的处理能力，包括多生产者、多消费者场景的表现。
   - 在并发条件下，测试增加服务节点后的扩展性，确保吞吐量随节点增加而提升。
   - 测试消息队列在故障场景（如断网、宕机、队列超时等）下的恢复能力，确保系统能够迅速恢复并保证消息的完整性。

2. **WebSocket测试**
   - 测试WebSocket在不同大小的数据体下的消息发送和接收性能。
   - 在不同并发条件下，测试WebSocket连接的建立时间、吞吐量和响应时间。
   - 验证WebSocket消息的传递顺序和可靠性，确保消息不会丢失、乱序或重复。
   - 测试WebSocket连接的稳定性和恢复能力，确保在异常断开后能够重新建立连接。

3. **点对点和发布/订阅模式测试**
   - 验证点对点（P2P）通信模式下消息的可靠性，确保消息从单个发送者到单个接收者的准确传递。
   - 测试发布/订阅（Pub/Sub）模式下消息的广播性能，确保所有订阅者能够收到完整的消息。
   - 在多节点和高并发环境下，测试点对点与发布/订阅模式的消息传递效率和系统表现。

## 测试内容与用例说明

### 详细测试用例说明

以下测试用例提供了详细的步骤、输入数据、执行方法以及预期结果，以确保测试人员能够准确地执行测试并获取可靠的结果。

### 1. 消息队列测试

- **用例1：不同大小数据包的发送与接收**
  - **输入**：发送小至1KB、大至10MB的数据包。
  - **预期结果**：所有数据包均能被成功接收，且无丢失、延迟明显增加等异常。

- **用例2：顺序性、一致性和持久性验证**
  - **输入**：发送一组有序的消息并对其进行持久化。
  - **预期结果**：接收端收到的消息顺序与发送顺序一致，所有消息均被持久化且无重复。

- **用例3：多节点环境下的消息传递**
  - **输入**：在多节点集群中发送消息。
  - **预期结果**：所有节点均能正确接收到消息，顺序和一致性保持一致。

- **用例4：持久化后的吞吐量和响应时间测试**
  - **输入**：在启用持久化的情况下，以不同负载发送消息。
  - **预期结果**：记录每种负载下的吞吐量和响应时间，分析其随负载变化的表现。

- **用例5：并发处理能力测试**
  - **输入**：模拟多生产者和多消费者同时对消息队列进行操作。
  - **预期结果**：系统能够正常处理所有请求，且吞吐量符合预期。

- **用例6：扩展性测试**
  - **输入**：增加服务节点数量，并保持相同的并发负载。
  - **预期结果**：随着节点数量增加，系统的吞吐量相应提升。

- **用例7：故障恢复能力测试**
  - **输入**：模拟网络断开、系统宕机等故障。
  - **预期结果**：系统能够迅速恢复，且消息不丢失、不重复。

### 2. WebSocket测试

- **用例1：不同数据体积下的消息传递**
  - **输入**：通过WebSocket发送小至1KB、大至5MB的数据包。
  - **预期结果**：消息能够成功传递，无显著延迟或丢失。

- **用例2：并发条件下的连接建立与响应**
  - **输入**：模拟高并发条件下同时建立多个WebSocket连接。
  - **预期结果**：记录连接建立时间、吞吐量和响应时间，确保在高并发条件下性能符合预期。

- **用例3：消息传递顺序与可靠性验证**
  - **输入**：发送一系列有序消息。
  - **预期结果**：接收端收到的消息顺序与发送顺序一致，无丢失或重复。

- **用例4：连接稳定性和恢复能力测试**
  - **输入**：模拟WebSocket连接的异常断开情况。
  - **预期结果**：连接能够自动重新建立，且消息继续传递。

### 3. 点对点和发布/订阅模式测试

- **用例1：点对点通信的可靠性测试**
  - **输入**：通过点对点模式发送消息。
  - **预期结果**：消息能够准确地从发送者传递到接收者，无丢失。

- **用例2：发布/订阅模式的广播性能测试**
  - **输入**：通过发布/订阅模式发送消息，多名订阅者接收。
  - **预期结果**：所有订阅者均能收到消息，无遗漏或延迟过大。

- **用例3：高并发环境下的点对点与发布/订阅模式测试**
  - **输入**：在高并发环境下，通过点对点和发布/订阅模式发送大量消息。
  - **预期结果**：系统能够保持高效的消息传递，所有订阅者与接收者均能正常收到消息。

