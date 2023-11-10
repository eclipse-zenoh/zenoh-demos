package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.intoKeyExpr

class ZDeleteFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZDeleteFragment::javaClass.name
    }

    override fun startExample() {
        viewModel.zenohSession?.apply {
            "demo/example/zenoh-android-put".intoKeyExpr().onSuccess { keyExpr ->
                keyExpr.use {
                    console.append("Deleting resources matching '$keyExpr'...\n")
                    this.delete(keyExpr).res()
                        .onSuccess { resetState() }
                        .onFailure {
                            handleError(TAG, "Failed to perform DELETE", it)
                        }
                }
            }
        }
    }

    override fun stopExample() {}

}