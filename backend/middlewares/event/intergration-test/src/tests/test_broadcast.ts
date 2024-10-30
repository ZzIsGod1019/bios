import { MessageTargetKind, newMessage } from 'asteroid-mq-sdk';
import { getNode } from '../api';
import { NON_BLOCKING_TOPIC } from '../consts';

interface TestMessage {
    "data": string
}
type MessageType = TestMessage | 'quit';

const createMessage = (data: string) => newMessage<MessageType>(
    {
        "data": data
    }, {
    topic: NON_BLOCKING_TOPIC,
    subjects: ["event/test_broadcast"],
    targetKind: MessageTargetKind.Available
});

const createQuitMessage = () => newMessage<MessageType>(
    'quit', {
    topic: NON_BLOCKING_TOPIC,
    subjects: ["event/test_broadcast"],
    targetKind: MessageTargetKind.Available
});


const TEST_BROADCAST_MESSAGE_COUNT = 100;
const TEST_RECEIVER_COUNT = 100;
export default async () => {
    const hostNode = await getNode();
    const recvNode = async (_: number) => {
        const node = await getNode();
        const ep = await node.createEndpoint(NON_BLOCKING_TOPIC, ["event/test_broadcast"]);
        const process = new Promise((resolve, reject) => {
            (
                async () => {
                    try {
                        for await (const message of ep.messages()) {
                            if (message !== undefined) {
                                message.received();
                                const payload = message.json<MessageType>();
                                if (payload === 'quit') {
                                    resolve(undefined);
                                    break;
                                }
                                message.processed();
                            } else {
                                resolve(undefined);
                                break;
                            }
                        }
                    } catch (error) {
                        reject(error);
                    }
                }
            )();
        });
        return process;
    }
    const tasks = [];
    for (let i = 0; i < TEST_RECEIVER_COUNT; i++) {
        const task = recvNode(i)
        tasks.push(task);
    }

    console.timeLog("broadcast", "start_broadcast");
    for (let i = 0; i < TEST_BROADCAST_MESSAGE_COUNT; i++) {
        await hostNode.sendMessage(createMessage(`message-${i}`));
    }
    console.timeLog("broadcast", "end");
    await hostNode.sendMessage(createQuitMessage());
    await Promise.all(tasks);
};