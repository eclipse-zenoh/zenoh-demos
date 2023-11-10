package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.publication.Publisher
import kotlinx.coroutines.*
import kotlinx.coroutines.Dispatchers.IO
import kotlinx.coroutines.Dispatchers.Main

class ZPubFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZPubFragment::javaClass.name
    }

    private var publisher: Publisher? = null
    private var keyExpr: KeyExpr? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        viewModel.zenohSession?.apply {
            "demo/example/zenoh-android-pub".intoKeyExpr().onSuccess { key ->
                keyExpr = key
                this.declarePublisher(key).res().onSuccess { pub ->
                    publisher = pub
                    GlobalScope.launch(IO) {
                        var idx = 0
                        while (exampleIsRunning) {
                            delay(1000)
                            val payload = "Pub from Android!"
                            withContext(Main) {
                                console.append(
                                    "Putting Data ('$keyExpr': '[${
                                        idx.toString().padStart(4, ' ')
                                    }] $payload')...\n"
                                )
                            }
                            pub.put(payload).res()
                            idx++
                        }
                    }
                }.onFailure {
                    handleError(TAG, "Failed to launch publisher", it)
                }
            }
        }
    }

    override fun stopExample() {
        exampleIsRunning = false
        publisher?.undeclare()
        keyExpr?.close()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}
