package main

import (
	"log"
	"os"

	"github.com/joho/godotenv"
)

func loadEnvVars() string {
	// init
	err := godotenv.Load()
	if err != nil {
		log.Fatal("Error loading .env file")
	}

	return os.Getenv("API_URL")
}

func main() {
	apiURL := loadEnvVars()
	// Call API and get all QSOs
	getQSOs(apiURL)
}
