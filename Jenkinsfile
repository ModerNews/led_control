pipeline {
  agent any

  stages {
    stage('Checkout repository') {
      steps {
        checkout scm
      }
    }

    stage('Build Docker Image') {
      steps {
        sh 'docker build -t led_control .' 
      }
    }

    stage('Run Docker Image') {
      steps {
        sh 'docker run -d led_control'
      }
    }
  }
}
