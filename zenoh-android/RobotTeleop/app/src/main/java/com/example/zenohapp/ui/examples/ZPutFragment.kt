package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.intoKeyExpr

class ZPutFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZPutFragment::javaClass.name
    }


    override fun startExample() {
        viewModel.zenohSession?.apply {
            "demo/example/zenoh-android-put".intoKeyExpr().onSuccess { key ->
                key.use {
                    val value = "Put from Android!"
                    this.put(key, value)
                        .res()
                        .onSuccess {
                            console.append("Putting Data ('$key': '$value')...\n")
                            resetState()
                        }
                }
            }.onFailure {
                handleError(TAG, "Failed to perform PUT", it)
            }
        }
    }

    override fun stopExample() {}
}