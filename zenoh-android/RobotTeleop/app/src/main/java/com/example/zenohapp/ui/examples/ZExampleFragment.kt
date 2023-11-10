package com.example.zenohapp.ui.examples

import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.fragment.app.Fragment
import androidx.lifecycle.ViewModelProvider
import com.example.zenohapp.R
import com.example.zenohapp.ZenohViewModel
import com.example.zenohapp.databinding.FragmentExampleBinding

abstract class ZExampleFragment : Fragment() {

    private var _binding: FragmentExampleBinding? = null
    private val binding get() = _binding!!

    protected var exampleIsRunning: Boolean = false
    protected lateinit var viewModel: ZenohViewModel
    protected lateinit var console: TextView
    private lateinit var button: Button

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        _binding = FragmentExampleBinding.inflate(inflater, container, false)
        val root: View = binding.root
        console = binding.console
        button = binding.controlButton

        viewModel = ViewModelProvider(requireActivity()).get(ZenohViewModel::class.java)

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

    protected fun handleError(tag: String, errorMsg: String, error: Throwable) {
        Log.e(tag, "$errorMsg: $error")
        Toast.makeText(
            activity,
            errorMsg,
            Toast.LENGTH_SHORT
        ).show()
    }

    protected fun resetState() {
        exampleIsRunning = false
        button.text = getString(R.string.start)
    }
}
