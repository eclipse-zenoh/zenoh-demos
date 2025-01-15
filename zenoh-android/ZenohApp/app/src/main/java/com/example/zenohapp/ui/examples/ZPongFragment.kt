package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.pubsub.Publisher
import io.zenoh.pubsub.Subscriber
import io.zenoh.qos.CongestionControl
import io.zenoh.qos.QoS
import io.zenoh.sample.Sample

class ZPongFragment : ZExampleFragment() {

    private var publisher: Publisher? = null
    private var subscriber: Subscriber<Unit>? = null

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExprPing = "test/ping".intoKeyExpr().getOrThrow()
        val keyExprPong = "test/pong".intoKeyExpr().getOrThrow()

        publisher = session.declarePublisher(keyExprPong, qos = QoS(CongestionControl.BLOCK, express = true)).getOrThrow()
        subscriber = session.declareSubscriber(keyExprPing, callback = { sample: Sample -> publisher!!.put(sample.payload).getOrThrow() }).getOrThrow()

        writeToConsole("Starting ZPong...")
    }

    override fun stopExample() {
        writeToConsole("Stopped ZPong.")
        publisher?.close()
        subscriber?.close()
        exampleIsRunning = false
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}
