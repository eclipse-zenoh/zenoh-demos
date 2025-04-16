package com.example.zenohdemo.ui.examples

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import io.zenoh.bytes.ZBytes
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.qos.CongestionControl
import io.zenoh.qos.QoS
import kotlinx.coroutines.*
import kotlinx.coroutines.channels.Channel

class ZPingFragment : ZExampleFragment() {

    private val payloadSize = 8
    private val warmup = 1
    private val n = 100
    private val data = ByteArray(payloadSize)

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        for (i in 0..<payloadSize) {
            data[i] = (i % 10).toByte()
        }
        return super.onCreateView(inflater, container, savedInstanceState)
    }

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExprPing = "test/ping".intoKeyExpr().getOrThrow()
        val keyExprPong = "test/pong".intoKeyExpr().getOrThrow()

        val sub = session.declareSubscriber(keyExprPong, Channel()).getOrThrow()
        val publisher = session.declarePublisher(keyExprPing, qos = QoS(CongestionControl.BLOCK, express = true)).getOrThrow()

        val payload = ZBytes.from(data)
        val samples = arrayListOf<Long>()

        // -- warmup --
        writeToConsole("Warming up for $warmup...")
        val startTime = System.currentTimeMillis()
        while (System.currentTimeMillis() - startTime < warmup) {
            publisher.put(payload).getOrThrow()
            runBlocking { sub.receiver.receive() }
        }

        for (x in 0..n ) {
            val writeTime = System.nanoTime()
            publisher.put(payload).getOrThrow()
            runBlocking { sub.receiver.receive() }
            val ts = (System.nanoTime() - writeTime) / 1_000 //convert to microseconds
            samples.add(ts)
        }

        for (x in samples.withIndex()) {
            writeToConsole("$payloadSize bytes: seq=${x.index} rtt=${x.value}µs lat=${x.value / 2}µs")
        }

        sub.close()
        publisher.close()
    }

    override fun stopExample() {
        exampleIsRunning = false
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}
