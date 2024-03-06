package com.example.zenohapp.ui.examples

import android.util.Log
import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.prelude.SampleKind
import io.zenoh.publication.Publisher
import io.zenoh.queryable.Query
import io.zenoh.queryable.Queryable
import io.zenoh.sample.Sample
import io.zenoh.subscriber.Subscriber
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Dispatchers.Main
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import org.apache.commons.net.ntp.TimeStamp

class ZPongFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZPongFragment::javaClass.name
    }

    private var publisher: Publisher? = null
    private var subscriber: Subscriber<Channel<Sample>>? = null
    private var pingKeyExpr: KeyExpr? = null
    private var pongKeyExpr: KeyExpr? = null
    private var running = false

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        viewModel.zenohSession?.apply {
            pingKeyExpr = "test/ping".intoKeyExpr().getOrThrow()
            pongKeyExpr = "test/pong".intoKeyExpr().getOrThrow()

            publisher = this.declarePublisher(pongKeyExpr!!).res().getOrThrow()
            subscriber = this.declareSubscriber(pingKeyExpr!!).res().getOrThrow()

            GlobalScope.launch(Dispatchers.IO) {
                withContext(Main) {
                    console.append(">> Subscriber started listening for pings... \n")
                }
                running = true
                subscriber!!.receiver!!.apply {
                    val iterator = this.iterator()
                    var count = 0
                    while (iterator.hasNext() && running) {
                        publisher!!.put("Pong").res()
                        if (count > 999) {
                            withContext(Main) {
                                console.append(">>Received 1000 pings...\n")
                            }
                            count = 0
                        } else {
                            count++
                        }
                    }
                }
            }
        }
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun stopExample() {
        GlobalScope.launch(Dispatchers.IO) {
            withContext(Main) {
                console.append(">> Stopped listening for pings. \n")
            }
        }
        running = false
        pingKeyExpr?.close()
        pongKeyExpr?.close()
        publisher?.close()
        subscriber?.close()
    }

}