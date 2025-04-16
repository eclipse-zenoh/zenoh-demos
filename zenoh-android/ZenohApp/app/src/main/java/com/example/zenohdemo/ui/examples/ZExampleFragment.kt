package com.example.zenohdemo.ui.examples

import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.lifecycle.ViewModelProvider
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.example.zenohdemo.R
import com.example.zenohdemo.ZenohViewModel
import com.example.zenohdemo.databinding.FragmentExampleBinding
import com.example.zenohdemo.ui.examples.utils.ConsoleAdapter
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

abstract class ZExampleFragment : Fragment() {

    private var _binding: FragmentExampleBinding? = null
    private val binding get() = _binding!!

    protected var exampleIsRunning: Boolean = false
    protected lateinit var viewModel: ZenohViewModel
    protected lateinit var button: Button
    private lateinit var consoleView: RecyclerView
    private val consoleAdapter: ConsoleAdapter = ConsoleAdapter()

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        _binding = FragmentExampleBinding.inflate(inflater, container, false)
        val root: View = binding.root
        consoleView = binding.recyclerView
        button = binding.controlButton

        consoleView.layoutManager = LinearLayoutManager(context)
        consoleView.adapter = consoleAdapter

        viewModel = ViewModelProvider(requireActivity())[ZenohViewModel::class.java]

        button.setText(R.string.start)
        button.setOnClickListener {
            toggleExample()
        }

        return root
    }

    private fun toggleExample() {
        if (!exampleIsRunning) {
            exampleIsRunning = true
            button.text = getString(R.string.stop)
            startExample()
        } else {
            exampleIsRunning = false
            button.text = getString(R.string.start)
            stopExample()
        }
    }

    protected abstract fun startExample()

    protected abstract fun stopExample()

    @OptIn(DelicateCoroutinesApi::class)
    protected fun writeToConsole(text: String) {
        GlobalScope.launch(Dispatchers.IO) {
            withContext(Dispatchers.Main) {
                consoleAdapter.addNewEntry(text)
                consoleView.scrollToPosition(consoleAdapter.itemCount - 1)
            }
        }
    }

    protected fun handleError(tag: String, errorMsg: String, error: Throwable) {
        Log.e(tag, "$errorMsg: $error")
        Toast.makeText(
            activity,
            errorMsg,
            Toast.LENGTH_SHORT
        ).show()
    }

    @OptIn(DelicateCoroutinesApi::class)
    protected fun resetState() {
        exampleIsRunning = false
        GlobalScope.launch(Dispatchers.IO) {
            withContext(Dispatchers.Main) {
                button.text = getString(R.string.start)
            }
        }
    }
}
