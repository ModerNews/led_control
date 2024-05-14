pipeline {
  agent any

  stages {
    stage('Checkout repository') {
      steps {
        checkout scm
      }
    }

    stage('Build & Start Image') {
      agent { dockerfile true }
      steps {
        sh led_control
      }
    }
  }
}
