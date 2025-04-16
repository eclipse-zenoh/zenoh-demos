package com.example.zenohdemo.ui.examples

import io.zenoh.bytes.ZBytes
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.pubsub.Publisher
import kotlinx.coroutines.*
import kotlinx.coroutines.Dispatchers.IO
import kotlinx.coroutines.Dispatchers.Main

class ZPubFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZPubFragment::javaClass.name
    }

    private var publisher: Publisher? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "demo/example/zenoh-android-pub".intoKeyExpr().getOrThrow()
        val publisher = session.declarePublisher(keyExpr).getOrThrow()
        GlobalScope.launch(IO) {
            var idx = 0
            while (exampleIsRunning) {
                delay(1000)
                val payload = ZBytes.from("Pub from Android!")
                withContext(Main) {
                    writeToConsole(
                        "Putting Data ('$keyExpr': '[${
                            idx.toString().padStart(4, ' ')
                        }] $payload')..."
                    )
                }
                publisher.put(payload)
                idx++
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
