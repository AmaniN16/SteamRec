from bs4 import BeautifulSoup

# Import csv module
import csv

# Import regex
import re

import requests
import xml.etree.ElementTree as ET

def scrape_steam_profile(steam_id):
    url = f"https://steamcommunity.com/id/{steam_id}/games?xml=1"
    response = requests.get(url)
    
    if response.status_code != 200:
        print(f"Failed to retrieve data. HTTP Status code: {response.status_code}")
        return
    
    root = ET.fromstring(response.text)
    
    steam_id = root.find('steamID').text
    games = []
    
    for game in root.findall('.//game'):
        app_id = game.find('appID').text
        name = game.find('name').text
        
        hours_on_record = game.find('hoursOnRecord')
        if hours_on_record is not None:
            hours_on_record = hours_on_record.text
        else:
            hours_on_record = "N/A"
        
        games.append({
            'name': name,
            'app_id': app_id,
            'hours_on_record': hours_on_record
        })
    
    games.sort(key=lambda game: game['name'])

    return games

def generate_csv(games):
    # URL of the website to be scraped for the current search query
    url = 'https://store.steampowered.com/search/?filter=topsellers'

    # Send a GET request to the specified URL
    response = requests.get(url)

    # Get the content of the downloaded page and save in a variable
    page_content = response.text
    page_content

    # Convert the file to a beautiful soup file
    doc = BeautifulSoup(page_content, 'html.parser')

    # List of search filters

    # , 'mostplayed', 'newreleases', 'upcomingreleases'

    # Create a CSV file to store the scraped data
    with open('games_mine.csv', mode='w', newline='', encoding='utf-8') as file:
        writer = csv.writer(file)
        writer.writerow(['Name', 'Published_Date', 'Price', 'Reviews', 'Hours_on_Record', 'App_Id', 'Genre', 'Developer'])

        count = 0

        # Loop through each search query
        # URL of the website to be scraped for the current search query
        url = f'https://store.steampowered.com/search/?filter=topsellers'

        # Send a GET request to the specified URL
        response = requests.get(url)

        # Parse the HTML content of the page using BeautifulSoup
        webpage = BeautifulSoup(response.content, 'html.parser')

        # Find the total number of pages
        total_pages = int(webpage.find('div', {'class': 'search_pagination_right'}).find_all('a')[-2].text)

        print("Scraping data...")

        # Loop through each page and extract the relevant information
        for page in range(1, total_pages + 1):
            # Send a GET request to the specified URL
            response = requests.get(url + '&page=' + str(page))

            # Parse the HTML content of the page using BeautifulSoup
            doc = BeautifulSoup(response.content, 'html.parser')

            # Find all the games on the page
            page_games = doc.find_all('div', {'class': 'responsive_search_name_combined'})

            # Loop through each game and extract the relevant information
            for game in page_games:
                name = game.find('span', {'class': 'title'}).text
                hours_on_record = 0

                for profile_game in games:
                    if profile_game['name'] == name:
                        hours_on_record = profile_game['hours_on_record']
                        count += 1

                published_date = game.find('div', {'class': 'col search_released responsive_secondrow'}).text.strip()

                discount_price_elem = game.find('div', {'class': 'discount_final_price'})
                discount_price = discount_price_elem.text.strip() if discount_price_elem else 'N/A'

                # Extract review information using regular expressions
                review_summary = game.find('span', {'class': 'search_review_summary'})
                reviews_html = review_summary['data-tooltip-html'] if review_summary else 'N/A'

                id_elem = game.parent
                app_id = id_elem['data-ds-appid'] if id_elem else 'N/A'

                appdetails_req = requests.get(f"http://store.steampowered.com/app/{app_id}")
                
                app_doc = BeautifulSoup(appdetails_req.content, 'html.parser')
                soup_data = app_doc.find('div', {'id': 'genresAndManufacturer'})

                if soup_data is not None:
                    print("Scraping game data with id {0}.".format(app_id))
                    # get game details, it's in the first block

                    genre_elem = soup_data.find('span')

                    if genre_elem is not None:
                        genres = []
                        for genre in genre_elem.findAll('a'):
                            genres.append(genre.text)

                        genres_string = ','.join(genre for genre in genres)

                            
                        print(genres_string)

                        dev_elem = soup_data.find('div', {'class': 'dev_row'})
                        developer = dev_elem.find('a')

                        dev_string = developer.text
                        
                        print(dev_string)

                        # Use regular expressions to extract the percentage of reviews
                        match = re.search(r'(\d{2})', reviews_html)
                        reviews_percentage = float(match.group(1))/100 if match else 'N/A'

                        # Write the extracted information to the CSV file
                        writer.writerow([name, published_date, discount_price, reviews_percentage, hours_on_record, app_id, genres_string, dev_string])
                

                if count == len(games):
                    return

steam_id = "Amoneyyy"
games = scrape_steam_profile(steam_id)
generate_csv(games)