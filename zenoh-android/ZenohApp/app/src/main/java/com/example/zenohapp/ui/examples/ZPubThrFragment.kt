package com.example.zenohapp.ui.examples

import io.zenoh.bytes.ZBytes
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.pubsub.Publisher
import io.zenoh.qos.CongestionControl
import io.zenoh.qos.Priority
import io.zenoh.qos.QoS
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch

class ZPubThrFragment : ZExampleFragment() {

    private val payloadSize = 8
    private val payload: ZBytes
    private val messagesNumber = 100000

    init {
        val data = ByteArray(payloadSize)
        for (i in 0..<payloadSize) {
            data[i] = (i % 10).toByte()
        }
        payload = ZBytes.from(data)
    }

    private var publisher: Publisher? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!

        val qos = QoS(
            congestionControl = CongestionControl.BLOCK,
            priority = Priority.DATA,
        )

        val keyExpr = "test/thr".intoKeyExpr().getOrThrow()
        publisher = session.declarePublisher(keyExpr, qos = qos).getOrThrow()

        writeToConsole(
            "Publisher declared on $keyExpr. Starting the throughput test" +
                    "with a payload size of $payloadSize with $messagesNumber messages and a quality of service of $qos..."
        )

        var count: Long = 0
        var start = System.currentTimeMillis()
        val number = messagesNumber.toLong()

        GlobalScope.launch(Dispatchers.IO) {
            while (exampleIsRunning) {
                publisher!!.put(payload).getOrThrow()
                if (count < number) {
                    count++
                } else {
                    val throughput = count * 1000 / (System.currentTimeMillis() - start)
                    writeToConsole("$throughput msgs/s")
                    count = 0
                    start = System.currentTimeMillis()
                }
            }
        }
    }

    override fun stopExample() {
        exampleIsRunning = false
        publisher?.undeclare()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}
