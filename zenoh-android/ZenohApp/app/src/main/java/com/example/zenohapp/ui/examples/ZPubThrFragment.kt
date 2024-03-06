package com.example.zenohapp.ui.examples

import android.util.Log
import io.zenoh.keyexpr.KeyExpr
import io.zenoh.keyexpr.intoKeyExpr
import io.zenoh.prelude.Encoding
import io.zenoh.prelude.KnownEncoding
import io.zenoh.publication.Publisher
import io.zenoh.value.Value
import kotlinx.coroutines.*
import kotlinx.coroutines.Dispatchers.IO
import kotlinx.coroutines.Dispatchers.Main

class ZPubThrFragment : ZExampleFragment() {
    companion object {
        private val TAG = ZPubThrFragment::javaClass.name
    }

    private var publisher: Publisher? = null
    private var keyExpr: KeyExpr? = null

    @OptIn(DelicateCoroutinesApi::class)
    override fun startExample() {
        val size = 8
        val sampleSize = 1000000

        val data = ByteArray(size)
        for (i in 0..<size) {
            data[i] = (i % 10).toByte()
        }
        val value = Value(data, Encoding(KnownEncoding.EMPTY))

        viewModel.zenohSession?.apply {
            "test/thr".intoKeyExpr().onSuccess { key ->
                keyExpr = key
                this.declarePublisher(key).res().onSuccess { pub ->
                    publisher = pub
                    GlobalScope.launch(IO) {
                        pub.use {
                            Log.i(TAG, "Publisher declared on test/thr.")
                            withContext(Main) {
                                console.append("Publisher declared on test/thr.\n")
                            }
                            var count = 0
                            var startTime = System.currentTimeMillis()
                            while (exampleIsRunning) {
                                pub.put(value).res()
                                if (count < sampleSize) {
                                    count++
                                } else {
                                    val elapsedTime = System.currentTimeMillis()
                                    val msgsPerSec = (count / (elapsedTime - startTime)) * 1_000
                                    Log.i(TAG, "$msgsPerSec msgs/sec")
                                    withContext(Main) {
                                        console.append("$msgsPerSec msgs/sec\n")
                                    }
                                    count = 0
                                    startTime = System.currentTimeMillis()
                                }
                            }
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
