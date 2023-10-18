package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.sample.Sample
import io.zenoh.subscriber.Subscriber
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Dispatchers.Main
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class ZSubFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZSubFragment::javaClass.name
    }

    private var subscriber: Subscriber<Channel<Sample>>? = null
    private var keyExpr: KeyExpr? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        viewModel.zenohSession?.apply {
            "demo/example/**".intoKeyExpr().onSuccess { key ->
                keyExpr = key
                console.append("Declaring Subscriber on '$key'...\n")
                this.declareSubscriber(key).res().onSuccess { sub ->
                    subscriber = sub
                    GlobalScope.launch(Dispatchers.IO) {
                        sub.receiver?.apply {
                            val iterator = this.iterator()
                            while (iterator.hasNext()) {
                                val sample = iterator.next()
                                withContext(Main) {
                                    console.append(">> [Subscriber] Received ${sample.kind} ('${sample.keyExpr}': '${sample.value}')\n")
                                }
                            }
                        }
                    }
                }.onFailure {
                    handleError(TAG, "Failed to launch subscriber", it)
                }
            }
        }
    }

    override fun stopExample() {
        subscriber?.undeclare()
        keyExpr?.close()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        stopExample()
    }
}