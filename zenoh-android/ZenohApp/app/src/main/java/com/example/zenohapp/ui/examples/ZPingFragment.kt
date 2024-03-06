package com.example.zenohapp.ui.examples

import android.util.Log
import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.publication.Publisher
import io.zenoh.sample.Sample
import io.zenoh.subscriber.Subscriber
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext

class ZPingFragment : ZExampleFragment() {
    private var publisher: Publisher? = null
    private var subscriber: Subscriber<Channel<Sample>>? = null
    private var pingKeyExpr: KeyExpr? = null
    private var pongKeyExpr: KeyExpr? = null
    private var running = false
    companion object {
        private val TAG = ZPingFragment::javaClass.name
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        viewModel.zenohSession?.apply {
            pingKeyExpr = "test/ping".intoKeyExpr().getOrThrow()
            pongKeyExpr = "test/pong".intoKeyExpr().getOrThrow()

            publisher = this.declarePublisher(pingKeyExpr!!).res().getOrThrow()
            subscriber = this.declareSubscriber(pongKeyExpr!!).res().getOrThrow()
            running = true
            GlobalScope.launch(Dispatchers.IO) {
                while (running) {
                    val measurements = mutableListOf<Long>()

                    for (i in 0..999) {
                        runBlocking {
                            val pubTime = System.nanoTime()
                            publisher!!.put("01234567").res().getOrThrow()
                            subscriber!!.receiver!!.receive()
                            val subTime = System.nanoTime()
                            measurements.add((subTime - pubTime) / 1_000_000)
                        }
                    }
                    withContext(Dispatchers.Main) {
                        val ping = String.format("%.3f", measurements.average())
                        Log.i(TAG, "Ping =  $ping ms")
                        console.append(">>Ping = $ping ms...\n")
                    }
                }
            }
        }
    }

    override fun stopExample() {
        running = false
        pingKeyExpr?.close()
        pongKeyExpr?.close()
        publisher?.close()
        subscriber?.close()
    }
}