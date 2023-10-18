package com.example.zenohapp.ui.examples

import io.zenoh.query.Reply
import io.zenoh.selector.intoSelector
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Dispatchers.Main
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.time.Duration

class ZGetFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZGetFragment::javaClass.name
        val timeout: Duration = Duration.ofMillis(1000)
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        viewModel.zenohSession?.apply {
            "demo/example/**".intoSelector().onSuccess { selector ->
                selector.use {
                    console.append("Sending Query '$it'...\n")
                    this.get(selector).timeout(timeout).res().onSuccess {
                        GlobalScope.launch(Dispatchers.IO) {
                            it?.iterator()?.apply {
                                while (hasNext()) {
                                    val reply = next()
                                    if (reply is Reply.Success) {
                                        withContext(Main) {
                                            console.append("Received ('${reply.sample.keyExpr}': '${reply.sample.value}')\n")
                                        }
                                    } else {
                                        reply as Reply.Error
                                        withContext(Main) {
                                            console.append("Received (ERROR: '${reply.error}')\n")
                                        }
                                    }
                                }
                                resetState()
                            }
                        }
                    }.onFailure {
                        handleError(TAG, "Failed to perform GET", it)
                    }
                }
            }
        }
    }

    override fun stopExample() {}
}