package com.example.zenohapp.ui.examples

import io.zenoh.keyexpr.intoKeyExpr

class ZDeleteFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZDeleteFragment::javaClass.name
    }

    override fun startExample() {
        val session = viewModel.zenohSession!!
        val keyExpr = "demo/example/zenoh-android-put".intoKeyExpr().getOrThrow()
        writeToConsole("Deleting resources matching '$keyExpr'...")
        session.delete(keyExpr)
            .onSuccess { resetState() }
            .onFailure {
                handleError(TAG, "Failed to perform DELETE", it)
            }
    }

    override fun stopExample() {}

}