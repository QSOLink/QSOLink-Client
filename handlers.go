package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"

	"QsolinkClient/qso"
)

func getQSOs(apiURL string) {

	resp, err := http.Get(apiURL)
	if err != nil {
		log.Fatal(err)
	}
	defer resp.Body.Close()

	var contacts []qso.Contact

	if err := json.NewDecoder(resp.Body).Decode(&contacts); err != nil {
		log.Fatal(err)
	}

	for _, contact := range contacts {
		fmt.Println(contact.ID, contact.Callsign, contact.Band, contact.Mode, contact.Name, contact.DateOn)
	}

}

func addQSO(apiURL string) {
	// this function should add a QSO to the API using the POST method.
	// Required fields from the Contact struct are:
	// Callsign, Band, Mode, DateOn, TimeOn
	// the DateOn should be today's date UTC in the format YYYY-MM-DD
	// the TimeOn should be the time the QSO was received in the format HH:MM:SS.SSS
	// the rest of the fields are optional and will be set to empty strings if not provided
	// except for qsls and qslr which should be set to false
}
