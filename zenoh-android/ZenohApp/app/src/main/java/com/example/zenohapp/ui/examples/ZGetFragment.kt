package com.example.zenohapp.ui.examples

import io.zenoh.query.intoSelector
import io.zenoh.sample.SampleKind
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import java.time.Duration

class ZGetFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZGetFragment::javaClass.name
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val session = viewModel.zenohSession!!
        val selector = "demo/example/**".intoSelector().getOrThrow()

        writeToConsole("Sending Query '$selector'...")
        session.get(selector, channel = Channel(), timeout = Duration.ofMillis(1000))
            .onSuccess { channel ->
                GlobalScope.launch(Dispatchers.IO) {
                    for (reply in channel) {
                        reply.result.onSuccess { sample ->
                            when (sample.kind) {
                                SampleKind.PUT -> writeToConsole("Received ('${sample.keyExpr}': '${sample.payload}')")
                                SampleKind.DELETE -> writeToConsole("Received (DELETE '${sample.keyExpr}')")
                            }
                        }.onFailure { error ->
                            writeToConsole("Received (ERROR: '${error.message}')")
                        }
                    }
                    resetState()
                }
            }.onFailure {
            handleError(TAG, "Failed to perform GET", it)
        }
    }

    override fun stopExample() {}
}